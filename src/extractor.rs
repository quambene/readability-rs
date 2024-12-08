use crate::dom;
use crate::scorer;
use crate::scorer::{Scorer, DEFAULT_SCORER};
use anyhow::anyhow;
use html5ever::{parse_document, serialize};
use html5ever::{tendril::stream::TendrilSink, ParseOpts};
use log::{debug, trace};
use markup5ever_rcdom::{RcDom, SerializableHandle};
#[cfg(feature = "reqwest")]
use reqwest;
use scorer::Candidate;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::default::Default;
use std::io::Read;
use std::path::Path;
#[cfg(feature = "reqwest")]
use std::time::Duration;
use url::Url;

#[derive(Debug)]
pub struct Product {
    pub title: String,
    pub content: String,
    pub text: String,
}

#[cfg(feature = "reqwest")]
pub fn scrape(url: &str) -> Result<Product, anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::new(30, 0))
        .build()?;
    let mut res = client.get(url).send()?;
    if res.status().is_success() {
        let url = Url::parse(url)?;
        let product = extract(&mut res, &url)?;
        Ok(product)
    } else {
        Err(anyhow!("Can't fetch {url}"))
    }
}

/// Extract text with a custom [`Scorer`].
pub fn extract_with_scorer<R>(
    input: &mut R,
    url: &Url,
    scorer: &Scorer,
) -> Result<Product, anyhow::Error>
where
    R: Read,
{
    let mut dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(input)?;

    if !dom.errors.is_empty() {
        return Err(anyhow!("Can't parse document: {:?}", dom.errors));
    }

    let mut title = String::new();
    let mut candidates = BTreeMap::new();
    let mut nodes = BTreeMap::new();
    let handle = dom.document.clone();
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

/// Extract text with the default [`Scorer`].
pub fn extract<R>(input: &mut R, url: &Url) -> Result<Product, anyhow::Error>
where
    R: Read,
{
    extract_with_scorer(input, url, &DEFAULT_SCORER)
}
