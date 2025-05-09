use crate::runners::wgpu_runner::run_compute_shader;
use crate::runners::example::run;
pub mod runners;

pub const SHADER: &[u8] = include_bytes!(env!("my_shader.spv"));

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
