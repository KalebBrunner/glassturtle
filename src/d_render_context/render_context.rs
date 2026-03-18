use std::sync::Arc;

use vulkano::{
    pipeline::{GraphicsPipeline, graphics::viewport::Viewport},
    render_pass::{Framebuffer, RenderPass},
    swapchain::Swapchain,
    sync::GpuFuture,
};

pub struct RenderContext {
    pub window: Arc<glfw::PWindow>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}
