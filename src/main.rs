// use crate::runners::example::run;
use crate::runners::wgpu_runner::run_compute_shader;
pub mod runners;

pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

fn main() {
    env_logger::init();
    run_compute_shader();
}
