use futures_intrusive::channel::shared::oneshot_channel;
use std::{borrow::Cow, collections::btree_map::Keys, convert::TryInto, default, str::FromStr};
use wgpu::util::DeviceExt;

use crate::SHADER;

const OVERFLOW: u32 = 0xffffffff;

// pub async fn run_compute_shader() {
//     let words = if std::env::args().len() <= 1 {
//         let default = ["рыба".as_bytes(), "раб".as_bytes()];
//         default
//     } else {
//         std::env::args()
//             .skip(1)
//             .map(|s| u32::from_str(&s).expect("You must pass a list of two words!"))
//             .collect()
//     };
//     let steps = execute_gpu(words);
    
// println!("levenshtein distance is equal to:", disp_steps.join(", "));
// }

pub async fn run() {
    let numbers = if std::env::args().len() <= 1 {
        let default = vec![1, 2, 3, 4];
        println!("No numbers were provided, defaulting to {:?}", default);
        default
    } else {
        std::env::args()
            .skip(1)
            .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
            .collect()
    };

    let steps = execute_gpu(numbers);

    let disp_steps: Vec<String> = steps
        .iter()
        .map(|&n| match n {
            OVERFLOW => "OVERFLOW".to_string(),
            _ => n.to_string(),
        })
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

pub fn execute_gpu(numbers: Vec<u32>) -> Vec<u32> {
    let shader_code = SHADER;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    let adapter_options = &wgpu::RequestAdapterOptions::default();
    let adapter_future = instance.request_adapter(&adapter_options);
    let adapter = pollster::block_on(adapter_future).unwrap();

    let device_descriptor = wgpu::DeviceDescriptor::default();
    let device_future = adapter.request_device(&device_descriptor, None);
    let (device, queue) = pollster::block_on(device_future).unwrap();

    let descriptor = wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Owned(
            wgpu::util::make_spirv_raw(shader_code).to_vec().into(),
        )),
    };
    let shader_module = device.create_shader_module(descriptor);

    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SPIR-V Fragment Shader"),
        source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Owned(
            wgpu::util::make_spirv_raw(shader_code).to_vec().into(),
        )),
    });

    let slice_size = numbers.len() * std::mem::size_of::<u32>();
    let size = slice_size as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Storage Buffer"),
        contents: bytemuck::cast_slice(&numbers),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main_cs",
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
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
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(numbers.len() as u32, 1, 1); 
}

    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

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
