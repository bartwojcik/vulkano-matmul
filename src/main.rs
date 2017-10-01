#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;


use std::sync::Arc;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::QueuesIter;
use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;


fn select_physical_device<'a>(instance: &'a Arc<Instance>) -> PhysicalDevice<'a> {
    let device_index = 0;
    PhysicalDevice::from_index(&instance, device_index)
        .expect("Failed to get the specified physical device.")
}

fn get_device(physical_device: PhysicalDevice) -> (Arc<Device>, QueuesIter) {
    let queue_family = physical_device.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Couldn't find a graphical queue family.");

    Device::new(physical_device, &Features::none(), &DeviceExtensions::none(),
                [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
}

mod cs {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "src/shader/compute.glsl"]
    struct Dummy;
}

fn main() {
    const K: usize = 4;
    const M: usize = 4;
    const N: usize = 4;

    //    let mut a = vec![0f32; K * M];
    //    let mut b = vec![0f32; N * K];
    //    let mut c = vec![0f32; N * M];
    let mut a = [0f32; K * M];
    let mut b = [0f32; N * K];
    let mut c = [0f32; N * M];

    // TODO compare performance and generated assembly (bound checks) for array initialization
    // - normal index access
    // - unsafe index access
    // - iterators
    for (i, v) in a.iter_mut().enumerate() {
        *v = (i / K + i % K) as f32;
    }
    for (i, v) in b.iter_mut().enumerate() {
        *v = (i / N + i % N) as f32;
    }

    // Vulkan initialization
    let app_infos = app_info_from_cargo_toml!();
    let instance = Instance::new(Some(&app_infos), &InstanceExtensions::none(), None)
        .expect("Failed to create instance - no Vulkan implementations available.");
    let (device, mut queues) = get_device(select_physical_device(&instance));
    let queue = queues.next().unwrap();

    let buffer_a = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(),
                                                  a).expect("failed to create buffer");

    let buffer_b = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(),
                                                  b).expect("failed to create buffer");

    let buffer_c = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(),
                                                  c).expect("failed to create buffer");

    let shader = cs::Shader::load(device.clone())
        .expect("Failed to create shader module.");
    let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
        .expect("Failed to create compute pipeline."));

    let descriptor_set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        .add_buffer(buffer_a.clone()).unwrap()
        .add_buffer(buffer_b.clone()).unwrap()
        .add_buffer(buffer_c.clone()).unwrap()
        .build().unwrap()
    );

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
        .dispatch([M as u32, N as u32, 1],
                  compute_pipeline.clone(),
                  descriptor_set.clone(),
                  [K as u32, M as u32, N as u32]).unwrap()
        .build().unwrap();
    println!("Created command buffer");

    let finished = command_buffer.execute(queue.clone()).unwrap();
    println!("Submitted command buffer to queue");

    finished.then_signal_fence_and_flush().unwrap()
        .wait(None).unwrap();
    println!("Command buffer finished executing");

    let mat_c_results = buffer_c.read().unwrap();
    for (n, val) in mat_c_results.iter().enumerate() {
        let x = n % N;
        let y = n / N;
        let mut expected_value = 0f32;
        for i in 0..K {
            expected_value += a[y * K + i] * b[x + i * N];
        }
        assert_eq!(*val, expected_value);
    }
    println!("Success");
}
