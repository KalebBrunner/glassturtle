use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::shaders::mesh_vertex::MeshVertex;

pub struct Mesh {
    pub vertex_buffer: Subbuffer<[MeshVertex]>,
}

pub fn create_mesh(device: Arc<Device>) -> Mesh {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let vertices = make_cube();
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

    Mesh { vertex_buffer }
}

fn make_cube() -> Vec<MeshVertex> {
    let red = [1.0, 0.1, 0.1];
    let green = [0.1, 1.0, 0.1];
    let blue = [0.1, 0.3, 1.0];
    let yellow = [1.0, 0.9, 0.1];
    let cyan = [0.1, 1.0, 1.0];
    let magenta = [1.0, 0.1, 1.0];

    vec![
        // front z = 0.5
        v([-0.5, -0.5, 0.5], red),
        v([0.5, -0.5, 0.5], red),
        v([0.5, 0.5, 0.5], red),
        v([-0.5, -0.5, 0.5], red),
        v([0.5, 0.5, 0.5], red),
        v([-0.5, 0.5, 0.5], red),
        // back z = -0.5
        v([0.5, -0.5, -0.5], green),
        v([-0.5, -0.5, -0.5], green),
        v([-0.5, 0.5, -0.5], green),
        v([0.5, -0.5, -0.5], green),
        v([-0.5, 0.5, -0.5], green),
        v([0.5, 0.5, -0.5], green),
        // left x = -0.5
        v([-0.5, -0.5, -0.5], blue),
        v([-0.5, -0.5, 0.5], blue),
        v([-0.5, 0.5, 0.5], blue),
        v([-0.5, -0.5, -0.5], blue),
        v([-0.5, 0.5, 0.5], blue),
        v([-0.5, 0.5, -0.5], blue),
        // right x = 0.5
        v([0.5, -0.5, 0.5], yellow),
        v([0.5, -0.5, -0.5], yellow),
        v([0.5, 0.5, -0.5], yellow),
        v([0.5, -0.5, 0.5], yellow),
        v([0.5, 0.5, -0.5], yellow),
        v([0.5, 0.5, 0.5], yellow),
        // top y = 0.5
        v([-0.5, 0.5, 0.5], cyan),
        v([0.5, 0.5, 0.5], cyan),
        v([0.5, 0.5, -0.5], cyan),
        v([-0.5, 0.5, 0.5], cyan),
        v([0.5, 0.5, -0.5], cyan),
        v([-0.5, 0.5, -0.5], cyan),
        // bottom y = -0.5
        v([-0.5, -0.5, -0.5], magenta),
        v([0.5, -0.5, -0.5], magenta),
        v([0.5, -0.5, 0.5], magenta),
        v([-0.5, -0.5, -0.5], magenta),
        v([0.5, -0.5, 0.5], magenta),
        v([-0.5, -0.5, 0.5], magenta),
    ]
}

fn v(position: [f32; 3], color: [f32; 3]) -> MeshVertex {
    MeshVertex { position, color }
}
