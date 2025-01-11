use criterion::{criterion_group, criterion_main, Criterion};
use lithtml::Dom;

static HTML: &'static str = include_str!("./wikipedia-2020-12-21.html");

fn wikipedia(c: &mut Criterion) {
    let mut group = c.benchmark_group("wikipedia");
    group.sample_size(10);
    group.bench_function("wikipedia", |b| b.iter(|| Dom::parse(HTML).unwrap()));
    group.finish();
}

criterion_group!(benches, wikipedia);
criterion_main!(benches);
