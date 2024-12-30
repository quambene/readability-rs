use html5ever::{parse_document, serialize, ParseOpts};
use readability::{extract_content, ExtractOptions, RcDom, ScorerOptions, SerializableHandle};
use std::{fs::File, str::FromStr, time::Instant};
use tendril::TendrilSink;
use url::Url;

fn extract_options<'a>(max_candidate_parents: usize) -> ExtractOptions<'a> {
    ExtractOptions {
        parse_options: Default::default(),
        scorer_options: ScorerOptions {
            max_candidate_parents,
            ..Default::default()
        },
    }
}

fn main() {
    let mut html = File::open("benches/wikipedia.html").unwrap();
    let url = Url::from_str("https://en.wikipedia.org/wiki/Particle_physics").unwrap();
    let opts = extract_options(2);
    let mut dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut html)
        .unwrap();

    let start = Instant::now();

    let content = extract_content(&mut dom, &url, opts);

    let mut bytes = vec![];

    serialize(
        &mut bytes,
        &SerializableHandle::from(content.node.clone()),
        Default::default(),
    )
    .unwrap();

    let elapsed = start.elapsed();

    println!("readability.rs: {elapsed:?}");
}
