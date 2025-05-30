use levenshtein_shader::levenshtein;

pub mod runners;
pub use runners::wgpu_runner::levenshtein_gpu;

pub const WORDS_PADDING: usize = 64;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

// take usual levenshtein for comparison and run it on CPU
pub fn levenshtein_distance(words: &[&str]) -> Vec<u32> {
    let number_of_words = words.len();
    let mut words_byted: Vec<u32> = Vec::with_capacity(number_of_words * WORDS_PADDING);
    let mut result = vec![0; number_of_words * number_of_words];

    // convert words to fixed-length u32 arrays with padding
    for w in words {
        assert!(w.len() <= WORDS_PADDING, "word too long");
        words_byted.extend(w.chars().map(|c| c as u32));
        words_byted.extend(std::iter::repeat(0).take(WORDS_PADDING - w.len()));
    }

    // calculate distances for all pairs
    for pair_idx in 0..number_of_words {
        let start = pair_idx * WORDS_PADDING;
        for compared_word_index in 0..number_of_words {
            let compared_word_start = compared_word_index * WORDS_PADDING;
            let dist = levenshtein(&words_byted, start, compared_word_start);
            result[pair_idx * number_of_words + compared_word_index] = dist;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_levenshtein() {
        let words = ["kitten", "sitting"];
        let distances = levenshtein_distance(&words);
        assert_eq!(distances, vec![0, 3, 3, 0]);
    }

    #[test]
    fn test_empty_words() {
        let words = ["", "test"];
        let distances = levenshtein_distance(&words);
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
