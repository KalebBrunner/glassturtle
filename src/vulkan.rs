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

// pub fn init_vulkan(window: Arc<PWindow>) -> (Arc<Instance>, Arc<Surface>, Arc<Device>, Arc<Queue>) {
//     let required_extensions =
//         Surface::required_extensions(&window).expect("Failed to get required extensions");

//     let vulkan = init_vkinstance(required_extensions);
//     let (device, queue) = init_device(vulkan.clone());
//     let surface = init_surface(vulkan.clone(), window);
//     (vulkan, surface, device, queue)
// }

pub fn init_vkinstance(windowing_extensions: InstanceExtensions) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("failed to load Vulkan library");
    println!("Vulkan ver: {:?}", library.api_version());

    let extensions = collect_extensions(&library, windowing_extensions);
    let layers = collect_layers(&library);

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

fn collect_extensions(
    library: &Arc<VulkanLibrary>,
    minimum: InstanceExtensions,
) -> InstanceExtensions {
    let supported = library.supported_extensions();

    // Check for minimums
    if minimum.difference(&supported).count() > 0 {
        println!("Required extensions: {:?}", minimum);
        println!("Supported extensions: {:?}", supported);
        panic!("Windowing surface not supported by vulkan")
    }

    let additionally_desired = InstanceExtensions {
        ext_debug_utils: true,
        khr_get_surface_capabilities2: true,
        ext_swapchain_colorspace: true,
        ..InstanceExtensions::empty()
    };

    // Check for additional
    let unavailable = additionally_desired.difference(supported);
    if unavailable.count() > 0 {
        println!("Failed to get {:?}", unavailable);
        panic!("Failed to collect extensions")
    } else {
        let active = minimum.union(&additionally_desired);
        println!("Extensions: {:?}", active);
        println!("Extensions: minimum and all additional active");
        return active;
    }
}

fn collect_layers(library: &VulkanLibrary) -> Vec<String> {
    if USE_VALIDATION_LAYERS == false {
        println!("Validation layers disabled");
        return Vec::new();
    }

    let requested_layers: Vec<String> = VALIDATION_LAYERS
        .iter()
        .map(|layer| layer.to_string())
        .collect();

    let available_layers: Vec<String> = library
        .layer_properties()
        .expect("Failed to enumerate Vulkan layer properties")
        .map(|layer| layer.name().to_string())
        .collect();

    let missing_layers: Vec<&String> = requested_layers
        .iter()
        .filter(|requested| {
            !available_layers
                .iter()
                .any(|available| available == *requested)
        })
        .collect();

    if missing_layers.is_empty() {
        println!("Layers: {:?}", requested_layers);
        println!("Layers: requested only");
        return requested_layers;
    } else {
        println!("Layers Available: {:?}", available_layers);
        println!("Required Vulkan validation layer(s) not found: {missing_layers:?}");
        panic!("");
    }
}

pub fn init_device(vulkan: Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
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
        "Physical name: {:?} | Type: {:?}",
        physical_device.properties().device_name,
        physical_device.properties().device_type
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
        Err(err) => panic!("Could not build device: {:?}", err),
    }
}

pub fn init_surface(vulkan: Arc<Instance>, window: Arc<PWindow>) -> Arc<Surface> {
    Surface::from_window(vulkan, window).expect("failed to create surface")
}
