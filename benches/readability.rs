use criterion::{criterion_group, criterion_main, Criterion};
use readability::{extract, ExtractOptions, ScorerOptions};
use std::{fs::File, str::FromStr};
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

fn bench_extract(c: &mut Criterion) {
    let mut html = File::open("benches/wikipedia.html").unwrap();
    let url = Url::from_str("https://en.wikipedia.org/wiki/Particle_physics").unwrap();

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
