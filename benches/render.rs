use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_render::{render, Config, OutputFormat};

const SMALL_HTML: &str = include_str!("fixtures/small.html");
const MEDIUM_HTML: &str = include_str!("fixtures/medium.html");
const LARGE_HTML: &str = include_str!("fixtures/large.html");

fn fixtures() -> Vec<(&'static str, &'static str)> {
    vec![
        ("small", SMALL_HTML),
        ("medium", MEDIUM_HTML),
        ("large", LARGE_HTML),
    ]
}

fn bench_full_pipeline_png(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline/png");

    for (name, html) in fixtures() {
        group.throughput(Throughput::Bytes(html.len() as u64));

        group.bench_with_input(BenchmarkId::new("render", name), &html, |b, html| {
            b.iter(|| render(black_box(html), Config::new().width(800).height(600)).unwrap());
        });
    }

    group.finish();
}

fn bench_full_pipeline_pdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline/pdf");

    for (name, html) in fixtures() {
        group.throughput(Throughput::Bytes(html.len() as u64));

        group.bench_with_input(BenchmarkId::new("render", name), &html, |b, html| {
            b.iter(|| {
                render(
                    black_box(html),
                    Config::new()
                        .width(800)
                        .height(600)
                        .format(OutputFormat::Pdf),
                )
                .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    let html = MEDIUM_HTML;

    for scale in [1.0, 2.0, 3.0] {
        group.bench_with_input(
            BenchmarkId::new("png", format!("{scale}x")),
            &scale,
            |b, &scale| {
                b.iter(|| {
                    render(
                        black_box(html),
                        Config::new().width(800).height(600).scale(scale),
                    )
                    .unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("dimensions");
    let html = MEDIUM_HTML;

    let sizes = [
        ("400x300", 400, 300),
        ("800x600", 800, 600),
        ("1920x1080", 1920, 1080),
    ];

    for (name, width, height) in sizes {
        group.throughput(Throughput::Elements((width * height) as u64));

        group.bench_with_input(
            BenchmarkId::new("png", name),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| render(black_box(html), Config::new().width(w).height(h)).unwrap());
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_full_pipeline_png,
    bench_full_pipeline_pdf,
    bench_scaling,
    bench_dimensions
);
criterion_main!(benches);
