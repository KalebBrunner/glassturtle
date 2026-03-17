use glfw::PWindow;
use std::sync::Arc;
use vulkano::{instance::Instance, swapchain::Surface};

pub fn init_surface(vulkan: Arc<Instance>, window: Arc<PWindow>) -> Arc<Surface> {
    Surface::from_window(vulkan, window).expect("failed to create surface")
}
