use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct MeshVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

#[derive(BufferContents, Clone, Copy)]
#[repr(C)]
pub struct CameraUniform {
    pub world_to_clip: [[f32; 4]; 4],
}
