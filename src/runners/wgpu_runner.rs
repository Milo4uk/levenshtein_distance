use futures_intrusive::channel::shared::oneshot_channel;
use std::convert::TryInto;
use wgpu::util::DeviceExt;

use crate::SHADER;

// const OVERFLOW: u32 = 0xffffffff;

//fill the words up
//we will fill them so they are even in length
//padding by the longest word, but for now it's 64
const MAX: usize = 64;

pub fn run_compute_shader() {
    //if they provided less than 2 words => warning
    let words = if std::env::args().len() < 2 {
        let default = vec!["hip".to_owned(), "hop".to_owned()];
        println!("No words were provided, defaulting to {:?}", default);
        default
    } else {
        std::env::args().collect()
    };
    let metrics = pollster::block_on(execute_gpu(words));
    print!("Metrics: {:?}", metrics)
}

// pub async fn run() {
//     let numbers = if std::env::args().len() <= 1 {
//         let default = vec![1, 2, 3, 4];
//         println!("No numbers were provided, defaulting to {:?}", default);
//         default
//     } else {
//         std::env::args()
//             .skip(1)
//             .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
//             .collect()
//     };

//     let steps = execute_gpu(numbers);

//     let disp_steps: Vec<String> = steps
//         .iter()
//         .map(|&n| match n {
//             OVERFLOW => "OVERFLOW".to_string(),
//             _ => n.to_string(),
//         })
//         .collect();

//     println!("Steps: [{}]", disp_steps.join(", "));
// }

pub async fn execute_gpu(words: Vec<String>) -> Vec<u32> {
    let shader_code = SHADER;
    let mut words_byted: Vec<u8> = Vec::with_capacity(words.len() * MAX);

    // so, we fill the vector of byted words with zeroes to distinguish between words on the gpu side
    // other option: we could pass another vector with starting indices of each word?
    for w in &words {
        assert!(w.len() <= MAX, "word too long");
        words_byted.extend_from_slice(w.as_bytes());
        // fill it up with zeroes
        words_byted.extend(core::iter::repeat(0).take(MAX - w.len()));
    }

    let bytes: &[u8] = bytemuck::cast_slice(&words_byted);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let adapter = instance
        .request_adapter(&Default::default())
        .await
        .expect("failed to create adapter");

    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .await
        .expect("failed to create device");

    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SPIR-V Fragment Shader"),
        source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Owned(
            wgpu::util::make_spirv_raw(shader_code).to_vec().into(),
        )),
    });

    // double check logic later
    let slice_size = std::mem::size_of::<u32>() * words.len() / 2;
    let size = slice_size as wgpu::BufferAddress;

    // copy data from output buffer here
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let byte_words = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Words in bytes"),
        // pass the byted words to the gpu
        contents: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let out_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Final distance"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // layout none defaults to auto layout
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        // the name of the function to execute
        entry_point: "main_cs",
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
                resource: out_buffer.as_entire_binding(),
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
        cpass.dispatch_workgroups(1, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&out_buffer, 0, &staging_buffer, 0, size);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);

    let (sender, receiver) = oneshot_channel();
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    device.poll(wgpu::Maintain::Wait);

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
