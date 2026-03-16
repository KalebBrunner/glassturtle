use std::sync::Arc;

use vulkano::{
    buffer::{self, Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::vertex_input::Vertex,
};

#[derive(BufferContents, Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct MyTriangleVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

pub fn init_vertex_bufffer(device: Arc<Device>) {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new(
        device.clone(),
        Default::default(),
    ));

    // Before we can start creating and recording command buffers, we need a way of allocating
    // them. Vulkano provides a command buffer allocator, which manages raw Vulkan command
    // pools underneath and provides a safe interface for them.
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    // We now create a buffer that will store the shape of our triangle.
    let vertices = [
        MyTriangleVertex {
            position: [-0.5, -0.25],
            color: [1.0, 0.0, 0.0],
        },
        MyTriangleVertex {
            position: [0.0, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        MyTriangleVertex {
            position: [0.25, -0.1],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let vertex_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices,
    )
    .unwrap();
}
