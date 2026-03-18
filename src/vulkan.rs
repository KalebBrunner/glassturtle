use std::sync::Arc;

use glfw::PWindow;
use vulkano::{
    Version, VulkanLibrary,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        physical::PhysicalDevice,
    },
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::Surface,
};

const USE_VALIDATION_LAYERS: bool = true;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub fn init_vulkan(window: Arc<PWindow>) -> (Arc<Instance>, Arc<Surface>, Arc<Device>, Arc<Queue>) {
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");

    let vulkan = init_vkinstance(required_extensions);
    let (device, queue) = init_device(vulkan.clone());
    let surface = init_surface(vulkan.clone(), window);
    (vulkan, surface, device, queue)
}

fn init_vkinstance(extensions: InstanceExtensions) -> Arc<Instance> {
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

fn init_device(vulkan: Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
    let physical_device = init_physical_device(&vulkan);
    let (logical_device, mut queues) = init_logical_device(physical_device.clone());
    let queue = queues.next().unwrap();

    (logical_device, queue)
}

fn init_physical_device(vulkan: &Arc<Instance>) -> Arc<PhysicalDevice> {
    let device_id = 0;
    let physical_device = vulkan
        .enumerate_physical_devices()
        .unwrap()
        .nth(device_id)
        .expect("Selected Vulkan physical device not found");
    println!(
        "Physical name: {:?}",
        physical_device.properties().device_name
    );
    physical_device
}

fn init_logical_device(
    physical_device: Arc<PhysicalDevice>,
) -> (Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>) {
    let features = DeviceFeatures::empty();
    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    match Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: 0,
                ..Default::default()
            }],
            enabled_extensions: extensions,
            enabled_features: features,
            ..Default::default()
        },
    ) {
        Ok(d) => d,
        Err(err) => panic!("Couldn't build device: {:?}", err),
    }
}

fn init_surface(vulkan: Arc<Instance>, window: Arc<PWindow>) -> Arc<Surface> {
    Surface::from_window(vulkan, window).expect("failed to create surface")
}
