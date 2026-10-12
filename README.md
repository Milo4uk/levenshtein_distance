# levenshtein_distance

Levenshtein (edit) distance in Rust, with two backends: a plain CPU implementation and a GPU implementation that runs compute shaders written in Rust ([rust-gpu](https://github.com/Rust-GPU/rust-gpu), compiled to SPIR-V) through [wgpu](https://wgpu.rs).

The tool takes a list of words and prints (or saves as CSV) the full matrix of pairwise distances.

```
$ levenshtein_with_gpu compute "kitten" "sitting" --gpu
Using GPU acceleration

Distance Matrix:
           kitten sitting
  kitten        0       3
 sitting        3       0
```

## Features

- Classic dynamic-programming Levenshtein distance (insert, delete, substitute), O(n·m) time and memory per pair
- CPU backend (single-threaded) and GPU backend (compute shaders, many word pairs in parallel)
- Cross-platform GPU access through wgpu (Vulkan, Metal, DirectX)
- CLI with input from the command line or from a file, timing output, and CSV export

## Benchmarks

Average of 10 runs, measured end to end: building the full distance matrix for all words and writing it to a CSV file. Outliers were excluded. Timing uses `std::time::Instant`.

| Words | CPU | GPU |
|------:|----:|----:|
| 107 | 75 ms | 780 ms |
| 435 | 896 ms | 2.81 s |
| 823 | 2.95 s | 6.54 s |
| 1,646 | 12.64 s | 9.57 s |
| 3,292 | 43.98 s | 17.56 s |

Test machine: MacBook with a 1.4 GHz quad-core Intel Core i5, integrated Intel Iris Plus Graphics 645, 8 GB RAM.

What the numbers show:

- On small inputs the CPU is faster (about 10x at 107 words) because copying data to the GPU costs more than the computation itself.
- Somewhere between 823 and 1,646 words the GPU overtakes the CPU. At 3,292 words it is about 2.5x faster.
- CPU time grows roughly quadratically with the number of words, GPU time grows much more slowly.

Notes: the CPU version is single-threaded, and results come from an integrated GPU, so a dedicated GPU may behave differently. The `criterion` benchmarks in `benches/` are experimental: they matched manual timing on small inputs but gave unstable results on large ones, so the table above uses manual timing.

## Requirements

- Rust toolchain as pinned in `rust-toolchain.toml` (an older nightly, `cargo 1.71.0-nightly`, 2023-05-23)
- One of the following graphics APIs available on the system: Vulkan, Metal, or DirectX
- A GPU, for the `--gpu` mode (the CPU mode works without it)

The project is pinned to an old nightly toolchain, so newer toolchains may not build it without updating rust-gpu.

## Build

```sh
cargo build --release
```

The shader crate is compiled to SPIR-V by `build.rs` (via `SpirvBuilder`) during `cargo build`. The resulting `.spv` file appears under:

```
target/spirv-builder/<target>/release/deps/<shader name>.spv
```

The default target is `spirv-unknown-spv1.4`. The compiled shader is embedded into the program with `include_bytes!` (the `SHADER` constant).

## Usage

```sh
cargo run --release -- help
cargo run --release -- compute "kitten" "sitting"
cargo run --release -- compute --file test_data/small.txt --gpu --verbose --csv
```

Flags for `compute`:

| Flag | Description |
|------|-------------|
| `--file <path>` | Read words from a file (separated by spaces). Without it, words are taken from the command line. |
| `--gpu` | Compute on the GPU (default is the CPU) |
| `--verbose` | Print the execution time |
| `--csv` | Write the result to `distances.csv` instead of printing it to the terminal |

Limitations:

- Words must consist of ASCII characters
- In GPU mode, the number of words is limited by the capabilities of the GPU
- Words in a file must be separated by spaces

## Project structure

```
.
├── levenshtein_shader/   # no_std shader crate (dylib), compiled to SPIR-V
├── src/
│   ├── main.rs           # entry point
│   ├── cli.rs            # command-line parsing and dispatch
│   └── lib.rs            # library code: functions, structs, constants
├── benches/              # experimental criterion benchmarks and wgpu runner
├── test_data/            # sample word lists (small, medium, large)
├── build.rs              # builds the shader to SPIR-V
├── rust-toolchain.toml
└── .cargo/config.toml
```

The repository is a Cargo workspace. The shader crate uses `#![no_std]` and is built as a `dylib`.

## Stack

Rust, wgpu, rust-gpu (SPIR-V), `spirv-builder`, `pollster`, `tokio`, `futures-intrusive`, `bytemuck`, `clap`, `colored`, `anyhow`, `csv`, `criterion`.

## Status and ideas

This is a small project and is not actively maintained. Possible next steps:

- Tune thread and workgroup sizes for specific GPUs
- Send data to the GPU in batches to raise the maximum number of words
- Size buffers dynamically by the longest word
- Efficient handling of very long strings
- Non-ASCII input (Unicode)

## References

- [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
- [rust-gpu book](https://rust-gpu.github.io/rust-gpu/book/platform-support.html) and [repository](https://github.com/Rust-GPU/rust-gpu)
- [wgpu hello-compute example](https://github.com/gfx-rs/wgpu-rs/blob/master/examples/hello-compute/main.rs)
- [WebGPU Fundamentals](https://webgpufundamentals.org) and the [WebGPU specification](https://www.w3.org/TR/webgpu/)
