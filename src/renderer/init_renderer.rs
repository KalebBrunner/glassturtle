use std::sync::Arc;

use vulkano::{
    device::Device,
    swapchain::Surface,
    sync::{self, GpuFuture},
};

use crate::{
    renderer::frame_buffer::window_size_dependent_setup,
    renderer::{renderpass::init_renderpass, swapchain::init_swapchain},
};

pub fn init_renderer(surface: Arc<Surface>, device: Arc<Device>) {
    let (swapchain, swapchain_images) = init_swapchain(&surface, device.clone());
    let render_pass = init_renderpass(&device, swapchain);

    let mut framebuffers = window_size_dependent_setup(&swapchain_images, render_pass.clone());

    let mut recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(device.clone()).boxed());
}
