use criterion::{criterion_group, criterion_main, Criterion};
use diploma_project::{levenshtein_distance, levenshtein_gpu};

use pollster::FutureExt as _;

fn bench_levenshtein(c: &mut Criterion) {
    let mut group = c.benchmark_group("Levenshtein comparison");

    let test_cases = [
        ("small", vec!["kitten", "sitting", "book", "back"]),
        (
            "medium",
            vec!["intention", "execution", "development", "deployment"],
        ),
        (
            "large",
            vec![
                "pneumonoultramicroscopicsilicovolcanoconiosis",
                "pneumonoultramicroscopicsilicovolcanoconioses",
                "pseudopseudohypoparathyroidism",
                "supercalifragilisticexpialidocious",
            ],
        ),
    ];
    for (name, words) in &test_cases {
        group.bench_function(format!("CPU/{}", name), |b| {
            b.iter(|| levenshtein_distance(words))
        });

        group.bench_function(format!("GPU/{}", name), |b| {
            b.iter(|| levenshtein_gpu(words).block_on())
        });
    }
    group.finish();
}

criterion_group!(benches, bench_levenshtein);
criterion_main!(benches);
