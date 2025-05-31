use super::super::LevenshteinGPU;
use crate::WORDS_PADDING;
use futures_intrusive::channel::shared::oneshot_channel;
use std::convert::TryInto;

/// The function that calls wgpu and takes a vector of words to find the levenshtein distance for them and an instance of LevenshteinGPU.
/// It then returns a matrix of distances for each word, in a way of cartesian product.
pub async fn levenshtein_gpu(gpu: &LevenshteinGPU, words: &[&str]) -> Vec<u32> {
    let mut words_byted: Vec<u32> = Vec::new();
    // so, we fill the vector of byted words with zeroes to distinguish between words on the gpu side
    for w in words {
        assert!(w.len() <= WORDS_PADDING, "word too long");
        // make sure it's u32 cause your shader uses u32
        // just always use u32
        words_byted.extend(w.chars().map(|c| c as u32));
        // fill it up with zeroes
        words_byted.extend(core::iter::repeat(0).take(WORDS_PADDING - w.len()));
    }

    // BUFFER_ALIGNMENT is even with 4
    // make sure it's even with 4
    // we will do cartesian product of 'words'
    let slice_size = std::mem::size_of::<u32>() * (words.len() * words.len());
    let size = slice_size as wgpu::BufferAddress;

    gpu.queue
        .write_buffer(&gpu.storage_buffer, 0, bytemuck::cast_slice(&words_byted));

    // bindings in shader must match the bindings in pipeline
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &gpu.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu.storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: gpu.output_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&gpu.compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute levenshtein distance");
        cpass.dispatch_workgroups((words.len() as u32 + 64) / 64, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&gpu.output_buffer, 0, &gpu.staging_buffer, 0, size);

    gpu.queue.submit(Some(encoder.finish()));

    let buffer_slice = gpu.staging_buffer.slice(..);

    let (sender, receiver) = oneshot_channel();
    let _buffer_future = buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    let _ = gpu.device.poll(wgpu::PollType::Wait);

    if let Ok(()) = pollster::block_on(async {
        match receiver.receive().await.unwrap() {
            Ok(_) => return Ok(()),
            Err(err) => {
                eprintln!("Buffer mapping failed: {:?}", err);
                Err(())
            }
        }
    }) {
        let data = buffer_slice.get_mapped_range();
        let result = data
            .chunks_exact(4)
            .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
            .collect();
        drop(data);
        gpu.staging_buffer.unmap();
        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}
