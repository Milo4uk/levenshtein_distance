use crate::runners::wgpu_runner::{levenshtein_gpu, run_compute_shader};
pub mod runners;

pub const WORDS_PADDING: usize = 64;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

fn main() {
    env_logger::init();
    run_compute_shader();
}
