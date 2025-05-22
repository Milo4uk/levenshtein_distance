use futures_intrusive::channel::shared::oneshot_channel;
use std::convert::TryInto;
use wgpu::util::DeviceExt;

use crate::SHADER;

//fill the words up
//we will fill them so they are even in length
//padding by the longest word, but for now it's 64
const WORDS_PADDING: usize = 64;

/// Words shouls come in pairs, it prints an array of results for each pair
pub fn run_compute_shader() {
    // it reads them two from the start, the amount of words must be a multiple of 2 or it will just ignore the last one
    // we will call this func from smwh else later, for now words are this:
    let words = vec![
        "hip".to_owned(),
        "hop".to_owned(),
        "hip".to_owned(),
        "hop".to_owned(),
        "hip".to_owned(),
        "hop".to_owned(),
        "hipppppo".to_owned(),
        "hop".to_owned(),
    ];

    println!("Words are: {:?}", words);
    let metrics = pollster::block_on(execute_gpu(words));
    print!("Metrics: {:?}", metrics)
}

pub async fn execute_gpu(words: Vec<String>) -> Vec<u32> {
    let shader_code = SHADER;
    let mut words_byted: Vec<u32> = Vec::new();

    // so, we fill the vector of byted words with zeroes to distinguish between words on the gpu side
    for w in &words {
        assert!(w.len() <= WORDS_PADDING, "word too long");
        // make sure it's u32 cause your shader uses u32
        // just always use u32
        words_byted.extend(w.chars().map(|c| c as u32));
        // fill it up with zeroes
        words_byted.extend(core::iter::repeat(0).take(WORDS_PADDING - w.len()));
    }

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

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
            wgpu::util::make_spirv_raw(shader_code).to_vec().into(),
        )),
    });

    // BUFFER_ALIGNMENT is even with 4
    let num_of_pairs = words.len() / 2;
    // make sure it's even with 4
    let slice_size = std::mem::size_of::<u32>() * num_of_pairs;
    let size = slice_size as wgpu::BufferAddress;

    let byte_words = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Words in bytes"),
        // pass the byted words to the spir-v
        contents: bytemuck::cast_slice(&words_byted),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Final distance for pairs of words"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ 
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // layout none defaults to auto layout
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        // the name of the function to execute
        entry_point: Some("main_cs"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    // bindings in shader must match the bindings in pipeline
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: byte_words.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute levenshtein distance");
        // we will only use one workgroup for now, just to make it work
        cpass.dispatch_workgroups(num_of_pairs as u32, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);

    let (sender, receiver) = oneshot_channel();
    let _buffer_future = buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    let _ = device.poll(wgpu::PollType::Wait);

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
        staging_buffer.unmap();
        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}
