use crate::{
    dom::{Handle, NodeData, RcDom, SerializableHandle},
    error::ReadabilityError,
    html,
    scorer::{self, Scorer, ScorerOptions, TopCandidate},
    utils::{debug_candidate, debug_candidates},
};
use html5ever::{parse_document, serialize, tendril::stream::TendrilSink, ParseOpts};
use log::{debug, trace};
use scorer::Candidate;
use std::{cell::Cell, collections::BTreeMap, default::Default, io::Read, path::Path};
use url::Url;

#[derive(Debug)]
pub struct Readable {
    pub title: String,
    pub content: String,
    pub text: String,
}

#[derive(Debug)]
pub struct Content {
    pub node: Handle,
    pub title: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractOptions<'a> {
    pub parse_options: ParseOptions,
    pub scorer_options: ScorerOptions<'a>,
}

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub strict: bool,
}

/// Extract content from an HTML reader.
pub fn extract<R>(
    input: &mut R,
    url: &Url,
    opts: ExtractOptions,
) -> Result<Readable, ReadabilityError>
where
    R: Read,
{
    let mut dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(input)?;

    if opts.parse_options.strict && !dom.errors.borrow().is_empty() {
        return Err(ReadabilityError::ParseHtml(dom.errors.into_inner()));
    }

    let content = extract_content(&mut dom, url, opts);

    let mut bytes = vec![];

    serialize(
        &mut bytes,
        &SerializableHandle::from(content.node.clone()),
        Default::default(),
    )?;

    let mut text: String = String::new();

    extract_text(content.node.clone(), &mut text, true);

    let content_string = String::from_utf8(bytes).unwrap_or_default();

    debug!("Extracted title: {}", content.title);
    trace!("Extracted text: {text}");
    trace!("Extracted content: {content_string}");

    Ok(Readable {
        title: content.title,
        content: content_string,
        text,
    })
}

/// Extract content `Node` from DOM.
pub fn extract_content(dom: &mut RcDom, url: &Url, opts: ExtractOptions) -> Content {
    let mut title = String::new();
    let mut candidates = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    let handle = dom.document.clone();
    let scorer = Scorer::new(opts.scorer_options);

    scorer.preprocess(dom, handle.clone(), &mut title);
    scorer.find_candidates(Path::new("/"), handle.clone(), &mut candidates, &mut nodes);

    debug!("Found candidates: {}", candidates.values().len());
    trace!("Found candidates: {:?}", debug_candidates(&candidates));

    let top_candidate = scorer.find_top_candidate(&candidates).unwrap_or_else(|| {
        TopCandidate::new(
            "/",
            Candidate {
                node: handle.clone(),
                score: Cell::new(0.0),
            },
        )
    });

    debug!(
        "Found top candidate: {:?}",
        debug_candidate(top_candidate.candidate())
    );

    scorer.clean(
        dom,
        Path::new(top_candidate.id()),
        top_candidate.node().clone(),
        url,
        &candidates,
    );

    Content {
        node: top_candidate.node().clone(),
        title,
    }
}

/// Convert HTML to formatted text, including linebreaks and whitespaces.
pub fn extract_text(handle: Handle, text: &mut String, deep: bool) {
    let mut last_tag_name = None;

    for child in handle.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } => {
                text.push_str(contents.borrow().as_ref());
            }
            NodeData::Element { .. } => {
                if deep {
                    if let Some(tag_name) = last_tag_name {
                        if &tag_name == "p" {
                            text.push('\n');
                        }
                    }

                    extract_text(child.clone(), text, deep);

                    last_tag_name = html::get_tag_name(child.clone());
                }
            }
            _ => (),
        }
    }
}
