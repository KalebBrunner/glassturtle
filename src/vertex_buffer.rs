use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::shaders::MyTriangleVertex;

pub fn init_vertex_bufffer(
    device: Arc<Device>,
) -> (
    Arc<StandardCommandBufferAllocator>,
    Subbuffer<[MyTriangleVertex]>,
) {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    // We now create a buffer that will store the shape of our triangle.
    let vertices = [
        MyTriangleVertex {
            position: [0.50, 0.5],
            color: [0.0, 0.0, 1.0],
        },
        MyTriangleVertex {
            position: [0.0, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        MyTriangleVertex {
            position: [-0.5, 0.5],
            color: [1.0, 0.0, 0.0],
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

    (command_buffer_allocator, vertex_buffer)
}
