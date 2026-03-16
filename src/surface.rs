use std::sync::Arc;

use glfw::PWindow;
use vulkano::{instance::Instance, swapchain::Surface};

pub fn create_surface(instance: Arc<Instance>, window: Arc<PWindow>) -> Arc<Surface> {
    Surface::from_window(instance, window).expect("failed to create surface")
}
