use std::sync::Arc;

use glfw::PWindow;
use vulkano::{
    device::{Device, Queue},
    instance::Instance,
    swapchain::Surface,
};

use crate::b_vulkan::{
    init_device::init_device, init_surface::init_surface, init_vkinstance::init_vkinstance,
};

pub fn init_vulkan(window: Arc<PWindow>) -> (Arc<Instance>, Arc<Surface>, Arc<Device>, Arc<Queue>) {
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");

    let vulkan = init_vkinstance(required_extensions);
    let (device, queue) = init_device(vulkan.clone());
    let surface = init_surface(vulkan.clone(), window);
    (vulkan, surface, device, queue)
}
