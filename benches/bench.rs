use criterion::{criterion_group, criterion_main, Criterion};
use levenshtein_with_gpu::{levenshtein_distance_cpu, levenshtein_gpu, LevenshteinGPU};
use std::{fs, time::Duration};
use pollster::FutureExt as _;

fn load_test_cases() -> Vec<(String, Vec<String>)> {
    let sizes = ["large"];

    let size_small = 107;
    let size_medium = 435;
    let size_large = 1646;

    let gpu = match sizes.first().unwrap() {
        &"small" => pollster::block_on(LevenshteinGPU::new(size_small)),
        &"medium" => pollster::block_on(LevenshteinGPU::new(size_medium)),
        &"large" => pollster::block_on(LevenshteinGPU::new(size_large)),
        _ => todo!(),
    };

    sizes.iter().map(|size| {
        let path = format!("./test_data/{}.txt", size);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to load {}", path));
        
        let words: Vec<String> = content.lines()
            .flat_map(|line| line.split_whitespace())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        (size.to_string(), words)
    }).collect()
}

fn config_benchmarks() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_secs(3))
        .measurement_time(Duration::from_secs(100))
}

fn bench_levenshtein(c: &mut Criterion) {
    let test_cases = load_test_cases();
    let gpu = pollster::block_on(LevenshteinGPU::new(1646));

    for (size, words) in test_cases {
        let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let mut group = c.benchmark_group(&size);
 
        group.bench_function(format!("CPU/{}", &size), |b| {
            b.iter(|| levenshtein_distance_cpu(&word_refs))
        });

        group.bench_function(format!("GPU/{}", &size), |b| {
            b.iter(|| levenshtein_gpu(&gpu, &word_refs).block_on())
        });
        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = config_benchmarks();
    targets = bench_levenshtein
}
criterion_main!(benches);
