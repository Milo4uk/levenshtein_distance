pub mod runners;
pub use runners::wgpu_runner::levenshtein_gpu;

pub const WORDS_PADDING: usize = 64;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

fn levenshtein(a: &str, b: &str) -> u32 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut dp = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }

    dp[a_len][b_len]
}

pub fn levenshtein_distance_cpu(words: &[&str]) -> Vec<u32> {
    let mut results : Vec<u32>= vec![];

    for word1 in words {
        for word2 in words {
            results.push(levenshtein(word1, word2));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_levenshtein() {
        let words = ["kitten", "sitting"];
        let distances = levenshtein_distance_cpu(&words);
        assert_eq!(distances, vec![0, 3, 3, 0]);
    }

    #[test]
    fn test_empty_words() {
        let words = ["", "test"];
        let distances = levenshtein_distance_cpu(&words);
        assert_eq!(distances, vec![0, 4, 4, 0]);
    }
}

/// The struct for the levenshtein_distance func.
/// Saves up time by initialising only once.
pub struct LevenshteinGPU {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub staging_buffer: wgpu::Buffer,
    pub output_buffer: wgpu::Buffer,
    pub storage_buffer: wgpu::Buffer,
}

impl LevenshteinGPU {
    pub async fn new(words_len: usize) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let buffer_size =
            (words_len * words_len * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

        let adapter = instance
            .request_adapter(&Default::default())
            .await
            .expect("failed to create adapter");

        let (device, queue) = adapter
            .request_device(&Default::default())
            .await
            .expect("failed to create device");

        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SPIR-V Fragment Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Owned(
                wgpu::util::make_spirv_raw(SHADER).to_vec().into(),
            )),
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Word Storage"),
            size: (words_len * WORDS_PADDING * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Final distance for pairs of words"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            // the name of the function to execute
            entry_point: Some("main_cs"),
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        LevenshteinGPU {
            device,
            queue,
            compute_pipeline: compute_pipeline.clone(),
            bind_group_layout: compute_pipeline.clone().get_bind_group_layout(0),
            staging_buffer,
            output_buffer,
            storage_buffer,
        }
    }
}
