use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::shaders::struct_triangle::MyTriangleVertex;

pub fn init_vertex_buffer(
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
    let vertices = make_sine_ribbon();
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

fn make_sine_ribbon() -> Vec<MyTriangleVertex> {
    let mut vertices = Vec::new();

    let samples = 80;
    let thickness = 0.08f32;

    for i in 0..(samples - 1) {
        let t0 = i as f32 / (samples - 1) as f32;
        let t1 = (i + 1) as f32 / (samples - 1) as f32;

        // map t into x in [-0.9, 0.9]
        let x0 = -0.9 + 1.8 * t0;
        let x1 = -0.9 + 1.8 * t1;

        // manually compute sine wave
        let y0 = 0.35 * (x0 * 6.0).sin();
        let y1 = 0.35 * (x1 * 6.0).sin();

        // same color on top/bottom at each sample
        // so gradient runs along the wave, not across its width
        let c0 = [t0, 0.2, 1.0 - t0];
        let c1 = [t1, 0.2, 1.0 - t1];

        let top0 = MyTriangleVertex {
            position: [x0, y0 + thickness],
            color: c0,
        };
        let bottom0 = MyTriangleVertex {
            position: [x0, y0 - thickness],
            color: c0,
        };
        let top1 = MyTriangleVertex {
            position: [x1, y1 + thickness],
            color: c1,
        };
        let bottom1 = MyTriangleVertex {
            position: [x1, y1 - thickness],
            color: c1,
        };

        // quad -> 2 triangles
        vertices.push(top0);
        vertices.push(bottom0);
        vertices.push(bottom1);

        vertices.push(top0);
        vertices.push(bottom1);
        vertices.push(top1);
    }

    vertices
}
