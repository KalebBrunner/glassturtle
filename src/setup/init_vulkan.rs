use std::sync::Arc;

use glfw::PWindow;
use vulkano::{device::Device, swapchain::Surface};

use crate::setup::{init_device, init_surface, init_vulkan_instance::init_vkinstance};

pub fn init_vulkan(window: Arc<PWindow>) -> (Arc<Surface>, Arc<Device>) {
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");

    let vulkan = init_vkinstance(required_extensions);
    let device = init_device(vulkan.clone());
    let surface = init_surface(vulkan.clone(), window);
    (surface, device)
}
