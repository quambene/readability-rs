use crate::{
    dom,
    error::ReadabilityError,
    scorer::{self, Scorer, ScorerOptions, TopCandidate},
};
use html5ever::{parse_document, serialize, tendril::stream::TendrilSink, ParseOpts};
use log::{debug, trace};
use markup5ever_rcdom::{Handle, RcDom, SerializableHandle};
use scorer::Candidate;
use std::{cell::Cell, collections::BTreeMap, default::Default, io::Read, path::Path};
use url::Url;

#[derive(Debug)]
pub struct Product {
    pub title: String,
    pub content: String,
    pub text: String,
}

#[derive(Debug)]
pub struct Content {
    pub node: Handle,
    pub title: String,
}

#[derive(Debug, Default)]
pub struct ExtractOptions<'a> {
    pub parse_options: ParseOptions,
    pub scorer_options: ScorerOptions<'a>,
}

#[derive(Debug, Default)]
pub struct ParseOptions {
    pub strict: bool,
}

/// Extract content from an HTML reader.
pub fn extract<R>(
    input: &mut R,
    url: &Url,
    opts: ExtractOptions,
) -> Result<Product, ReadabilityError>
where
    R: Read,
{
    let mut dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(input)?;

    if opts.parse_options.strict && !dom.errors.is_empty() {
        return Err(ReadabilityError::ParseHtml(dom.errors));
    }

    let content = extract_content(&mut dom, url, opts);

    let mut bytes = vec![];

    serialize(
        &mut bytes,
        &SerializableHandle::from(content.node.clone()),
        Default::default(),
    )?;

    let mut text: String = String::new();

    dom::extract_text(content.node.clone(), &mut text, true);

    let content_string = String::from_utf8(bytes).unwrap_or_default();

    debug!("Extracted title: {}", content.title);
    debug!("Extracted text: {text}");
    debug!("Extracted content: {content_string}");

    Ok(Product {
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
    trace!(
        "Found candidates: {:?}",
        candidates
            .values()
            .map(|candidate| candidate.node.clone())
            .collect::<Vec<_>>()
    );

    let top_candidate = scorer.find_top_candidate(&candidates).unwrap_or_else(|| {
        TopCandidate::new(
            "/",
            Candidate {
                node: handle.clone(),
                score: Cell::new(0.0),
            },
        )
    });
    let node = top_candidate.node();

    debug!("Found top candidate: {node:?}");

    scorer.clean(
        dom,
        Path::new(top_candidate.id()),
        node.clone(),
        url,
        &candidates,
    );

    Content {
        node: node.clone(),
        title,
    }
}
