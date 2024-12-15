use crate::{
    dom,
    error::ReadabilityError,
    scorer::{self, Scorer, ScorerOptions},
};
use html5ever::{parse_document, serialize, tendril::stream::TendrilSink, ParseOpts};
use log::{debug, trace};
use markup5ever_rcdom::{RcDom, SerializableHandle};
use scorer::Candidate;
use std::{cell::Cell, collections::BTreeMap, default::Default, io::Read, path::Path};
use url::Url;

#[derive(Debug)]
pub struct Product {
    pub title: String,
    pub content: String,
    pub text: String,
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

/// Extract content and text with a custom [`Scorer`].
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

    let mut title = String::new();
    let mut candidates = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    let handle = dom.document.clone();
    let scorer = Scorer::new(opts.scorer_options);
    scorer.preprocess(&mut dom, handle.clone(), &mut title);
    scorer.find_candidates(Path::new("/"), handle.clone(), &mut candidates, &mut nodes);

    debug!("Found candidates: {}", candidates.values().len());
    trace!(
        "Found candidates: {:?}",
        candidates
            .values()
            .map(|candidate| candidate.node.clone())
            .collect::<Vec<_>>()
    );

    let mut id: &str = "/";
    let mut top_candidate: &Candidate = &Candidate {
        node: handle.clone(),
        score: Cell::new(0.0),
    };
    for (i, c) in candidates.iter() {
        let score = c.score.get() * (1.0 - scorer::get_link_density(c.node.clone()));
        c.score.set(score);
        if score <= top_candidate.score.get() {
            continue;
        }
        id = i;
        top_candidate = c;
    }
    let mut bytes = vec![];

    let node = top_candidate.node.clone();

    debug!("Found top candidate: {node:?}");

    scorer.clean(&mut dom, Path::new(id), node.clone(), url, &candidates);

    serialize(
        &mut bytes,
        &SerializableHandle::from(node.clone()),
        Default::default(),
    )
    .ok();
    let content = String::from_utf8(bytes).unwrap_or_default();

    let mut text: String = String::new();
    dom::extract_text(node.clone(), &mut text, true);

    debug!("Extracted title: {title}");
    debug!("Extracted content: {content}");
    debug!("Extracted text: {text}");

    Ok(Product {
        title,
        content,
        text,
    })
}
