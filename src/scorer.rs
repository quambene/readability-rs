use crate::dom;
use html5ever::{
    namespace_url, ns,
    tree_builder::{ElementFlags, NodeOrText, TreeSink},
    LocalName, QualName,
};
use lazy_static::lazy_static;
use markup5ever_rcdom::{
    Handle, Node,
    NodeData::{Comment, Doctype, Document, Element, ProcessingInstruction, Text},
    RcDom,
};
use regex::Regex;
use std::{borrow::Cow, cell::Cell, collections::BTreeMap, path::Path, rc::Rc};
use url::Url;

const PUNCTUATIONS_REGEX: &str = r"([、。，．！？]|\.[^A-Za-z0-9]|,[^0-9]|!|\?)";
// TODO: remove "comment" from unlikely candidates
const UNLIKELY_CANDIDATES: &str = "combx|comment|community|disqus|extra|foot|header|menu\
     |remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate\
     |pagination|pager|popup|tweet|twitter\
     |ssba";
const LIKELY_CANDIDATES: &str = "and|article|body|column|main|shadow\
                                              |content|hentry";
const POSITIVE_CANDIDATES: &str = "article|body|content|entry|hentry|main|page\
     |pagination|post|text|blog|story";
// TODO: remove "comment" and "com" from unlikely candidates
const NEGATIVE_CANDIDATES: &str = "combx|comment|com|contact|foot|footer|footnote\
     |masthead|media|meta|outbrain|promo|related\
     |scroll|shoutbox|sidebar|sponsor|shopping\
     |tags|tool|widget|form|textfield\
     |uiScale|hidden";
const BLOCK_CHILD_TAGS: [&str; 10] = [
    "a",
    "blockquote",
    "dl",
    "div",
    "img",
    "ol",
    "p",
    "pre",
    "table",
    "ul",
];
lazy_static! {
    static ref PUNCTUATIONS: Regex = Regex::new(PUNCTUATIONS_REGEX).unwrap();
    static ref LIKELY: Regex = Regex::new(LIKELY_CANDIDATES).unwrap();
    static ref UNLIKELY: Regex = Regex::new(UNLIKELY_CANDIDATES).unwrap();
    static ref POSITIVE: Regex = Regex::new(POSITIVE_CANDIDATES).unwrap();
    static ref NEGATIVE: Regex = Regex::new(NEGATIVE_CANDIDATES).unwrap();
}

#[derive(Clone)]
pub struct Candidate {
    pub node: Rc<Node>,
    pub score: Cell<f32>,
}

pub struct TopCandidate<'a> {
    id: Cow<'a, str>,
    candidate: Cow<'a, Candidate>,
}

impl<'a> TopCandidate<'a> {
    pub fn new(id: &'a str, candidate: Candidate) -> Self {
        Self {
            id: Cow::Borrowed(id),
            candidate: Cow::Owned(candidate),
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn candidate(&self) -> &Candidate {
        &self.candidate
    }

    pub fn node(&self) -> &Rc<Node> {
        &self.candidate.node
    }

    pub fn score(&self) -> &Cell<f32> {
        &self.candidate.score
    }
}

/// Distribution of the content score among parent nodes.
#[derive(Debug)]
pub enum CandidateScore {
    /// The same weight for all parent nodes.
    EqualWeight,
    /// The weight decreases with the level of the parent node.
    ///
    /// For example, a parent node will be weighted more than a grandparent.
    LevelWeight,
}

#[derive(Debug)]
pub struct ScorerOptions<'a> {
    /// The minimum word length of candidates.
    pub min_candidate_length: usize,
    /// The maximal number of parent nodes that will be traversed.
    pub max_candidate_parents: usize,
    /// Distribution of the content score among parent nodes.
    pub candidate_score: CandidateScore,
    pub punctuations: &'a Regex,
    pub unlikely_candidates: &'a Regex,
    pub likely_candidates: &'a Regex,
    pub positive_candidates: &'a Regex,
    pub positive_candidate_weight: f32,
    pub negative_candidates: &'a Regex,
    pub negative_candidate_weight: f32,
    pub block_child_tags: &'a [&'a str],
}

impl Default for ScorerOptions<'_> {
    fn default() -> Self {
        Self {
            min_candidate_length: 20,
            max_candidate_parents: 10,
            candidate_score: CandidateScore::EqualWeight,
            punctuations: &PUNCTUATIONS,
            likely_candidates: &LIKELY,
            unlikely_candidates: &UNLIKELY,
            positive_candidates: &POSITIVE,
            positive_candidate_weight: 25.0,
            negative_candidates: &NEGATIVE,
            negative_candidate_weight: 25.0,
            block_child_tags: &BLOCK_CHILD_TAGS,
        }
    }
}

pub struct Scorer<'a> {
    options: ScorerOptions<'a>,
}

impl<'a> Scorer<'a> {
    pub fn new(options: ScorerOptions<'a>) -> Self {
        Scorer { options }
    }

    pub fn preprocess(&self, dom: &mut RcDom, handle: Handle, title: &mut String) -> bool {
        if let Element {
            ref name,
            ref attrs,
            ..
        } = handle.clone().data
        {
            let tag_name = name.local.as_ref();
            match tag_name.to_lowercase().as_ref() {
                "script" | "link" | "style" => return true,
                "title" => dom::extract_text(handle.clone(), title, true),
                _ => (),
            }
            for name in ["id", "class"].iter() {
                if let Some(val) = dom::attr(name, &attrs.borrow()) {
                    if tag_name != "body"
                        && self.options.unlikely_candidates.is_match(&val)
                        && !self.options.likely_candidates.is_match(&val)
                    {
                        return true;
                    }
                }
            }
        }
        let mut useless_nodes = vec![];
        let mut paragraph_nodes = vec![];
        let mut br_count = 0;
        for child in handle.children.borrow().iter() {
            if self.preprocess(dom, child.clone(), title) {
                useless_nodes.push(child.clone());
            }
            let c = child.clone();
            match c.data {
                Element { ref name, .. } => {
                    let tag_name = name.local.as_ref();
                    if "br" == tag_name.to_lowercase() {
                        br_count += 1
                    } else {
                        br_count = 0
                    }
                }
                Text { ref contents } => {
                    let s = contents.borrow();
                    if br_count >= 2 && !s.trim().is_empty() {
                        paragraph_nodes.push(child.clone());
                        br_count = 0
                    }
                }
                _ => (),
            }
        }
        for node in useless_nodes.iter() {
            dom.remove_from_parent(node);
        }
        for node in paragraph_nodes.iter() {
            let name = QualName::new(None, ns!(), LocalName::from("p"));
            let p = dom.create_element(name, vec![], ElementFlags::default());
            dom.append_before_sibling(node, NodeOrText::AppendNode(p.clone()));
            dom.remove_from_parent(node);
            if let Text { ref contents } = node.clone().data {
                let text = contents.clone().into_inner().clone();
                dom.append(&p, NodeOrText::AppendText(text))
            }
        }
        false
    }

    /// Find candidate tags in DOM node, and distribute score among them.
    pub fn find_candidates(
        &self,
        node_id: &Path,
        handle: Handle,
        candidates: &mut BTreeMap<String, Candidate>,
        nodes: &mut BTreeMap<String, Rc<Node>>,
    ) {
        if let Some(id) = node_id
            .to_str()
            .map(|candidate_id| candidate_id.to_string())
        {
            nodes.insert(id, handle.clone());
        }

        if self.is_candidate(handle.clone()) {
            let content_score = self.calculate_content_score(handle.clone());

            let mut current_node_id = Some(node_id.to_path_buf());
            let mut level = 1;

            // Traverse all parent nodes and distribute content score.
            while let Some(current_id) = current_node_id {
                // Break traversal for performance reasons.
                if level > self.options.max_candidate_parents {
                    break;
                }

                if let Some(candidate) =
                    current_id.to_str().map(|id| id.to_string()).and_then(|id| {
                        // Only parent nodes are valid candidates.
                        if current_id != node_id {
                            self.find_or_create_candidate(Path::new(&id), candidates, nodes)
                        } else {
                            None
                        }
                    })
                {
                    let adjusted_content_score = match self.options.candidate_score {
                        CandidateScore::EqualWeight => content_score,
                        CandidateScore::LevelWeight => content_score / (level as f32),
                    };
                    candidate
                        .score
                        .set(candidate.score.get() + adjusted_content_score);

                    // Ignore candidates above the `body` node.
                    if dom::get_tag_name(candidate.node.clone()).as_deref() == Some("body") {
                        break;
                    }
                }

                current_node_id = current_id.parent().map(|pid| pid.to_path_buf());
                level += 1;
            }
        }

        for (i, child) in handle.children.borrow().iter().enumerate() {
            self.find_candidates(
                node_id.join(i.to_string()).as_path(),
                child.clone(),
                candidates,
                nodes,
            )
        }
    }

    // TODO: find top candidates with similar score.
    pub fn find_top_candidate(
        &self,
        candidates: &'a BTreeMap<String, Candidate>,
    ) -> Option<TopCandidate<'a>> {
        let mut top_candidate: Option<TopCandidate> = None;

        for (id, candidate) in candidates.iter() {
            let score = candidate.score.get() * (1.0 - get_link_density(candidate.node.clone()));
            candidate.score.set(score);

            if top_candidate
                .as_ref()
                .map_or(true, |top| score > top.candidate.score.get())
            {
                top_candidate = Some(TopCandidate {
                    id: Cow::Borrowed(id),
                    candidate: Cow::Borrowed(candidate),
                });
            }
        }

        top_candidate
    }

    pub fn clean(
        &self,
        dom: &mut RcDom,
        id: &Path,
        handle: Handle,
        url: &Url,
        candidates: &BTreeMap<String, Candidate>,
    ) -> bool {
        let mut useless = false;
        match handle.data {
            Document => (),
            Doctype { .. } => (),
            Text { ref contents } => {
                let s = contents.borrow();
                if s.trim().is_empty() {
                    useless = true
                }
            }
            Comment { .. } => useless = true,
            Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();
                match tag_name.to_lowercase().as_ref() {
                    "script" | "link" | "style" | "noscript" | "meta" | "h1" | "object"
                    | "header" | "footer" | "aside" => useless = true,
                    "form" | "table" | "ul" | "div" => {
                        useless = self.is_useless(id, handle.clone(), candidates)
                    }
                    "img" => useless = !fix_img_path(handle.clone(), url),
                    "a" => useless = !fix_anchor_path(handle.clone(), url),
                    _ => (),
                }
                dom::clean_attr("id", &mut attrs.borrow_mut());
                dom::clean_attr("class", &mut attrs.borrow_mut());
                dom::clean_attr("style", &mut attrs.borrow_mut());
            }
            ProcessingInstruction { .. } => unreachable!(),
        }
        let mut useless_nodes = vec![];
        for (i, child) in handle.children.borrow().iter().enumerate() {
            let pid = id.join(i.to_string());
            if self.clean(dom, pid.as_path(), child.clone(), url, candidates) {
                useless_nodes.push(child.clone());
            }
        }
        for node in useless_nodes.iter() {
            dom.remove_from_parent(node);
        }
        if dom::is_empty(handle) {
            useless = true
        }
        useless
    }

    fn calculate_content_score(&self, handle: Handle) -> f32 {
        let mut score: f32 = 1.0;
        let mut text = String::new();
        dom::extract_text(handle.clone(), &mut text, true);
        let mat = self.options.punctuations.find_iter(&text);
        score += mat.count() as f32;
        score += f32::min(f32::floor(text.chars().count() as f32 / 100.0), 3.0);
        score
    }

    fn get_class_weight(&self, handle: Handle) -> f32 {
        let mut weight: f32 = 0.0;
        if let Element {
            name: _, ref attrs, ..
        } = handle.data
        {
            for name in ["id", "class"].iter() {
                if let Some(val) = dom::attr(name, &attrs.borrow()) {
                    if self.options.positive_candidates.is_match(&val) {
                        weight += self.options.positive_candidate_weight
                    };
                    if self.options.negative_candidates.is_match(&val) {
                        weight -= self.options.negative_candidate_weight
                    }
                }
            }
        };
        weight
    }

    fn init_content_score(&self, handle: Handle) -> f32 {
        let tag_name = dom::get_tag_name(handle.clone()).unwrap_or_default();
        let score = match tag_name.as_ref() {
            "article" => 10.0,
            "div" => 5.0,
            "pre" | "td" | "blockquote" => 3.0,
            "address" | "ol" | "ul" | "dl" | "dd" | "dt" | "li" | "form" => -3.0,
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "th" => -5.0,
            _ => 0.0,
        };
        score + self.get_class_weight(handle.clone())
    }

    fn find_or_create_candidate(
        &self,
        id: &Path,
        candidates: &'a mut BTreeMap<String, Candidate>,
        nodes: &BTreeMap<String, Rc<Node>>,
    ) -> Option<&'a Candidate> {
        if let Some(id) = id.to_str().map(|id| id.to_string()) {
            if let Some(node) = nodes.get(&id) {
                if candidates.get(&id).is_none() {
                    candidates.insert(
                        id.clone(),
                        Candidate {
                            node: node.clone(),
                            score: Cell::new(self.init_content_score(node.clone())),
                        },
                    );
                }
                return candidates.get(&id);
            }
        }
        None
    }

    fn is_useless(
        &self,
        id: &Path,
        handle: Handle,
        candidates: &BTreeMap<String, Candidate>,
    ) -> bool {
        let tag_name = &dom::get_tag_name(handle.clone()).unwrap_or_default();
        let weight = self.get_class_weight(handle.clone());
        let score = id
            .to_str()
            .and_then(|id| candidates.get(id))
            .map(|c| c.score.get())
            .unwrap_or(0.0);
        if weight + score < 0.0 {
            return true;
        }
        let text_nodes_len = dom::text_children_count(handle.clone());
        let mut p_nodes: Vec<Rc<Node>> = vec![];
        let mut img_nodes: Vec<Rc<Node>> = vec![];
        let mut li_nodes: Vec<Rc<Node>> = vec![];
        let mut input_nodes: Vec<Rc<Node>> = vec![];
        let mut embed_nodes: Vec<Rc<Node>> = vec![];
        dom::find_node(handle.clone(), "p", &mut p_nodes);
        dom::find_node(handle.clone(), "img", &mut img_nodes);
        dom::find_node(handle.clone(), "li", &mut li_nodes);
        dom::find_node(handle.clone(), "input", &mut input_nodes);
        dom::find_node(handle.clone(), "embed", &mut embed_nodes);
        let p_count = p_nodes.len();
        let img_count = img_nodes.len();
        let li_count = li_nodes.len() as i32 - 100;
        let input_count = input_nodes.len();
        let embed_count = embed_nodes.len();
        let link_density = get_link_density(handle.clone());
        let content_length = dom::text_len(handle.clone());
        let para_count = text_nodes_len + p_count;

        if img_count > para_count + text_nodes_len {
            return true;
        }
        if li_count > para_count as i32 && tag_name != "ul" && tag_name != "ol" {
            return true;
        }
        if input_count as f32 > f32::floor(para_count as f32 / 3.0) {
            return true;
        }
        if content_length < 25 && (img_count == 0 || img_count > 2) {
            return true;
        }
        if weight < 25.0 && link_density > 0.2 {
            return true;
        }
        if (embed_count == 1 && content_length < 35) || embed_count > 1 {
            return true;
        }
        false
    }

    fn is_candidate(&self, handle: Handle) -> bool {
        let text_len = dom::text_len(handle.clone());
        if text_len < self.options.min_candidate_length {
            return false;
        }
        let n: &str = &dom::get_tag_name(handle.clone()).unwrap_or_default();
        match n {
            "p" => true,
            "div" | "article" | "center" | "section" => {
                !dom::has_nodes(handle.clone(), self.options.block_child_tags)
            }
            _ => false,
        }
    }
}

pub fn fix_img_path(handle: Handle, url: &Url) -> bool {
    let src = dom::get_attr("src", handle.clone());
    let s = match src {
        Some(src) => src,
        None => return false,
    };
    if !s.starts_with("//") && !s.starts_with("http://") && !s.starts_with("https://") {
        if let Ok(new_url) = url.join(&s) {
            dom::set_attr("src", new_url.as_str(), handle)
        }
    }
    true
}

pub fn fix_anchor_path(handle: Handle, url: &Url) -> bool {
    let src = dom::get_attr("href", handle.clone());
    let s = match src {
        Some(src) => src,
        None => return false,
    };
    if !s.starts_with("//") && !s.starts_with("http://") && !s.starts_with("https://") {
        if let Ok(new_url) = url.join(&s) {
            dom::set_attr("href", new_url.as_str(), handle)
        }
    }
    true
}

pub fn get_link_density(handle: Handle) -> f32 {
    let text_length = dom::text_len(handle.clone()) as f32;
    if text_length == 0.0 {
        return 0.0;
    }
    let mut link_length = 0.0;
    let mut links: Vec<Rc<Node>> = vec![];
    dom::find_node(handle.clone(), "a", &mut links);
    for link in links.iter() {
        link_length += dom::text_len(link.clone()) as f32;
    }
    link_length / text_length
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{debug_candidates, CandidateTag};
    use html5ever::{parse_document, tendril::TendrilSink};
    use std::{fs::File, io::Read};

    #[test]
    fn test_find_candidates_basic() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test Title</title></head>
            <body>
                <h1>Welcome</h1>
                <p>This is a test paragraph with more than 25 characters.</p>
            </body>
        </html>"#;
        let options = ScorerOptions::default();
        let scorer = Scorer::new(options);
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        assert!(dom.errors.is_empty(), "{:?}", dom.errors);

        let mut candidates = BTreeMap::new();
        let mut nodes = BTreeMap::new();

        scorer.find_candidates(Path::new("/"), dom.document, &mut candidates, &mut nodes);

        let tags = dbg!(debug_candidates(&candidates));

        assert_eq!(candidates.len(), 1);
        assert!(tags.contains(&CandidateTag::new("body", None, 1.0)));
    }

    #[test]
    fn test_find_candidates_comments() {
        let mut file = File::open("data/comments/input.html").unwrap();
        let mut html = String::new();
        file.read_to_string(&mut html).unwrap();

        let options = ScorerOptions {
            unlikely_candidates: &Regex::new(
                "combx|community|disqus|extra|foot|header|menu|remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate|pagination|pager|popup|tweet|twitter|ssba",
            )
            .unwrap(),
            negative_candidates: &Regex::new("combx|contact|foot|footer|footnote|masthead|media|meta|outbrain|promo|related|scroll|shoutbox|sidebar|sponsor|shopping|tags|tool|widget|form|textfield|uiScale|hidden").unwrap(),
            positive_candidates: &Regex::new("article|body|content|entry|hentry|main|page|pagination|post|blog|story").unwrap(),
            ..Default::default()
        };
        let scorer = Scorer::new(options);
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        assert!(dom.errors.is_empty(), "{:?}", dom.errors);

        let mut candidates = BTreeMap::new();
        let mut nodes = BTreeMap::new();

        scorer.find_candidates(Path::new("/"), dom.document, &mut candidates, &mut nodes);

        let tags = dbg!(debug_candidates(&candidates));

        assert_eq!(candidates.len(), 15);

        assert!(tags.contains(&CandidateTag::new("tbody", None, 6.0)));
        assert!(tags.contains(&CandidateTag::new("tr", Some("tr_2"), 6.0)));
        assert!(tags.contains(&CandidateTag::new("table", Some("table_2"), 6.0)));
        assert!(tags.contains(&CandidateTag::new("td", Some("td_0"), 9.0)));
        assert!(tags.contains(&CandidateTag::new("td", Some("td_1"), 5.0)));
        assert!(tags.contains(&CandidateTag::new("td", Some("td_2"), 5.0)));
        assert!(tags.contains(&CandidateTag::new("td", Some("td_3"), 5.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("comment_1"), 7.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("comment_2"), 7.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("comment_3"), 7.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("commtext_1"), 7.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("commtext_2"), 7.0)));
        assert!(tags.contains(&CandidateTag::new("div", Some("commtext_3"), 7.0)));
    }
}
