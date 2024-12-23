use criterion::{criterion_group, criterion_main, Criterion};
use readability::{extract, ExtractOptions, ScorerOptions};
use regex::Regex;
use std::{fs::File, str::FromStr, sync::OnceLock};
use url::Url;

static UNLIKELY_CANDIDATES: OnceLock<Regex> = OnceLock::new();
static NEGATIVE_CANDIDATES: OnceLock<Regex> = OnceLock::new();
static POSITIVE_CANDIDATES: OnceLock<Regex> = OnceLock::new();

fn extract_options<'a>(max_candidate_parents: usize) -> ExtractOptions<'a> {
    ExtractOptions { parse_options: Default::default(), scorer_options: ScorerOptions {
        max_candidate_parents,
         unlikely_candidates: UNLIKELY_CANDIDATES.get_or_init(|| {
             Regex::new(
                 "combx|community|disqus|extra|foot|header|menu|remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate|pagination|pager|popup|tweet|twitter|ssba",
             )
             .unwrap()
         }),
         negative_candidates: NEGATIVE_CANDIDATES.get_or_init(|| {
             Regex::new("combx|contact|foot|footer|footnote|masthead|media|meta|outbrain|promo|related|scroll|shoutbox|sidebar|sponsor|shopping|tags|tool|widget|form|textfield|uiScale|hidden").unwrap()
         }),
         positive_candidates: POSITIVE_CANDIDATES.get_or_init(|| {
             Regex::new("article|body|content|entry|hentry|main|page|pagination|post|blog|story").unwrap()
         }),
         ..Default::default()
     }}
}

fn bench_extract(c: &mut Criterion) {
    let mut html = File::open("benches/bench.html").unwrap();
    let url = Url::from_str("https://news.ycombinator.com/item?id=42200407").unwrap();

    c.bench_function("extract 2", |b| {
        b.iter(|| extract(&mut html, &url, extract_options(2)));
    });

    c.bench_function("extract 5", |b| {
        b.iter(|| extract(&mut html, &url, extract_options(5)));
    });

    c.bench_function("extract 10", |b| {
        b.iter(|| extract(&mut html, &url, extract_options(10)));
    });

    c.bench_function("extract 100", |b| {
        b.iter(|| extract(&mut html, &url, extract_options(100)));
    });
}

criterion_group!(name = benches; config = Criterion::default().sample_size(10); targets = bench_extract);
criterion_main!(benches);
