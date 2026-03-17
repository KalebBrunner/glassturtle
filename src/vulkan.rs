use std::sync::Arc;

use glfw::PWindow;
use vulkano::{
    Version, VulkanLibrary,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::Surface,
    sync::{self, GpuFuture},
};

use crate::{
    device::{init_logical_device, init_physical_device},
    frame_buffer::window_size_dependent_setup,
    renderpass::init_renderpass,
    swapchain::init_swapchain,
};

const USE_VALIDATION_LAYERS: bool = true;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub fn init_vulkan_instance(extensions: InstanceExtensions) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("failed to load Vulkan loader");
    println!("Highest Vulkan ver: {:?}", library.api_version());

    let mut layers = vec![];
    if USE_VALIDATION_LAYERS {
        layers = create_validation_layers(&library);
    }

    println!("Active layers: {:?}", layers);
    println!("Active extensions: {:?}", extensions);

    let create_info = InstanceCreateInfo {
        engine_name: Some("Glass Turtle Graphics".into()),
        engine_version: Version::V1_5,
        max_api_version: Some(Version::V1_5),
        enabled_layers: layers,
        enabled_extensions: extensions,
        ..InstanceCreateInfo::application_from_cargo_toml()
    };

    Instance::new(library, create_info).expect("failed to create Vulkan instance")
}

pub fn init_vulkan(window: Arc<PWindow>) {
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");
    let vulkan = init_vulkan_instance(required_extensions);
    let surface = Surface::from_window(vulkan.clone(), window).expect("failed to create surface");
    let physical_device = init_physical_device(&vulkan);
    let (logical_device, mut queues) = init_logical_device(physical_device.clone());
    let queue = queues.next().unwrap();

    let (swapchain, swapchain_images) = init_swapchain(&surface, logical_device.clone());
    let render_pass = init_renderpass(&logical_device, swapchain);
    let mut framebuffers = window_size_dependent_setup(&swapchain_images, render_pass.clone());
    let mut recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(logical_device.clone()).boxed());
}

fn create_validation_layers(library: &Arc<VulkanLibrary>) -> Vec<String> {
    let available_layer_names: Vec<String> = library
        .layer_properties()
        .unwrap()
        .map(|layer| layer.name().to_owned())
        .collect();

    let required_layer_names = VALIDATION_LAYERS.map(|layer| layer.to_owned()).to_vec();

    let missing_layer_names: Vec<_> = required_layer_names
        .iter()
        .filter(|req| !available_layer_names.iter().any(|avail| avail == *req))
        .collect();

    assert!(
        missing_layer_names.is_empty(),
        "Required Vulkan validation layer(s) not found: {:?}",
        missing_layer_names,
    );

    required_layer_names
}
