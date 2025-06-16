pub mod runners;
use levenshtein::levenshtein;
use csv::Writer;

pub use runners::wgpu_runner::levenshtein_gpu;

pub const WORDS_PADDING: usize = 32;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));


pub fn levenshtein_distance_cpu(words: &[&str]) -> Vec<u32> {
    let n = words.len();
    let mut results = vec![0; n * n];
    
    for i in 0..n {
        for j in 0..n {
            results[i * n + j] = levenshtein(words[i], words[j]) as u32;
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

    #[test]
    fn test_bananas() {
        let words = ["kitten", "kill", "bananas"];
        let distances = levenshtein_distance_cpu(&words);
        assert_eq!(distances, vec![0, 4, 7, 4, 0, 7, 7, 7, 0]);
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

#[derive(serde::Serialize)]
struct LevenshteinRecord {
    word_a: String,
    word_b: String,
    distance: u32,
}

pub fn save_to_csv(
    words: &[&str],
    distances: &[u32],
) {
    let mut writer = Writer::from_path("distances.csv").expect("failed to create .csv file");
    
    writer.write_record(&["first_word", "second_word", "distance"]).expect("failed to write to .csv file");
    
    for (i, word_a) in words.iter().enumerate() {
        for (j, word_b) in words.iter().enumerate() {
            if i < j {
                writer.serialize(LevenshteinRecord {
                    word_a: word_a.to_string(),
                    word_b: word_b.to_string(),
                    distance: distances[i * words.len() + j],
                }).expect("failed to write to .csv file");
            }
        }
    }

    writer.flush().expect("failed to write to .csv file");
}