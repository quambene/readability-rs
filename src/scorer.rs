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
const UNLIKELY_CANDIDATES: &str = "combx|comment|community|disqus|extra|foot|header|menu\
     |remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate\
     |pagination|pager|popup|tweet|twitter\
     |ssba";
const LIKELY_CANDIDATES: &str = "and|article|body|column|main|shadow\
                                              |content|hentry";
const POSITIVE_CANDIDATES: &str = "article|body|content|entry|hentry|main|page\
     |pagination|post|text|blog|story";
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

    pub fn node(&self) -> &Rc<Node> {
        &self.candidate.node
    }

    pub fn score(&self) -> &Cell<f32> {
        &self.candidate.score
    }
}

#[derive(Debug)]
pub struct ScorerOptions<'a> {
    pub punctuations: &'a Regex,
    pub unlikely_candidates: &'a Regex,
    pub likely_candidates: &'a Regex,
    pub positive_candidates: &'a Regex,
    pub negative_candidates: &'a Regex,
    pub block_child_tags: &'a [&'a str],
}

impl Default for ScorerOptions<'_> {
    fn default() -> Self {
        Self {
            punctuations: &PUNCTUATIONS,
            likely_candidates: &LIKELY,
            unlikely_candidates: &UNLIKELY,
            positive_candidates: &POSITIVE,
            negative_candidates: &NEGATIVE,
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

    pub fn find_candidates(
        &self,
        id: &Path,
        handle: Handle,
        candidates: &mut BTreeMap<String, Candidate>,
        nodes: &mut BTreeMap<String, Rc<Node>>,
    ) {
        if let Some(id) = id.to_str().map(|id| id.to_string()) {
            nodes.insert(id, handle.clone());
        }

        if self.is_candidate(handle.clone()) {
            let score = self.calc_content_score(handle.clone());
            if let Some(c) = id
                .parent()
                .and_then(|pid| self.find_or_create_candidate(pid, candidates, nodes))
            {
                c.score.set(c.score.get() + score)
            }
            if let Some(c) = id
                .parent()
                .and_then(|pid| pid.parent())
                .and_then(|gpid| self.find_or_create_candidate(gpid, candidates, nodes))
            {
                c.score.set(c.score.get() + score / 2.0)
            }
        }

        if self.is_candidate(handle.clone()) {
            let score = self.calc_content_score(handle.clone());
            if let Some(c) = id
                .to_str()
                .map(|id| id.to_string())
                .and_then(|id| candidates.get(&id))
            {
                c.score.set(c.score.get() + score)
            }
            if let Some(c) = id
                .parent()
                .and_then(|pid| pid.to_str())
                .map(|id| id.to_string())
                .and_then(|pid| candidates.get(&pid))
            {
                c.score.set(c.score.get() + score)
            }
            if let Some(c) = id
                .parent()
                .and_then(|p| p.parent())
                .and_then(|pid| pid.to_str())
                .map(|id| id.to_string())
                .and_then(|pid| candidates.get(&pid))
            {
                c.score.set(c.score.get() + score)
            }
        }

        for (i, child) in handle.children.borrow().iter().enumerate() {
            self.find_candidates(
                id.join(i.to_string()).as_path(),
                child.clone(),
                candidates,
                nodes,
            )
        }
    }

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

    fn calc_content_score(&self, handle: Handle) -> f32 {
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
                        weight += 25.0
                    };
                    if self.options.negative_candidates.is_match(&val) {
                        weight -= 25.0
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
        if text_len < 20 {
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
