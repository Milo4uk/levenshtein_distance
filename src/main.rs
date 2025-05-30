use crate::runners::wgpu_runner::levenshtein_gpu;
use diploma_project::LevenshteinGPU;
pub mod runners;

pub const WORDS_PADDING: usize = 64;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

fn main() {
    env_logger::init();
    run_compute_shader();
}

pub fn run_compute_shader() {
    let words = ["hip", "hop", "hip", "hop", "hip", "hop", "hipppppo", "hop"];
    let metrics = pollster::block_on(levenshtein_gpu(
        &pollster::block_on(LevenshteinGPU::new(words.len())),
        &words,
    ));
    print!("Metrics: {:?}", metrics)
}
