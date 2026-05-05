use std::sync::Arc;

use glfw::PWindow;
use vulkano::{
    Version, VulkanLibrary,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDevice,
    },
    instance::{
        Instance, InstanceCreateInfo, InstanceExtensions,
        debug::{DebugUtilsMessenger, DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo},
    },
    pipeline::graphics,
    swapchain::{Surface, SurfaceInfo},
};

const USE_VALIDATION_LAYERS: bool = true;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

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

    let vulkan = Instance::new(library, create_info).expect("failed to create Vulkan instance");
    let _debug_messenger = setup_debug_messenger(vulkan.clone());
    return vulkan;
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

fn setup_debug_messenger(instance: Arc<Instance>) -> DebugUtilsMessenger {
    if !USE_VALIDATION_LAYERS {
        panic!("Debug set without validation layers");
    }
    let debug_messenger = DebugUtilsMessenger::new(
        instance,
        DebugUtilsMessengerCreateInfo::user_callback(unsafe {
            DebugUtilsMessengerCallback::new(|severity, ty, data| {
                eprintln!("[Vulkan][{:?}][{:?}] {}", severity, ty, data.message);
            })
        }),
    )
    .expect("failed to create Vulkan debug messenger");
    return debug_messenger;
}

pub fn init_device(vulkan: Arc<Instance>, surface: Arc<Surface>) -> (Arc<Device>, Arc<Queue>) {
    let physical_device = init_physical_device(&vulkan);

    let (logical_device, mut queues) =
        init_logical_device(physical_device.clone(), surface.clone());
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
    if physical_device.supported_extensions().khr_swapchain == false {
        panic!("Device Extension: khr_Swapchain not available on this device");
    }
    physical_device
}

fn init_logical_device(
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
) -> (Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>) {
    let features = DeviceFeatures::empty();
    println!("Device features: {:?}", features);

    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    println!("Device extensions: {:?}", extensions);

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(index, queue_family)| {
            queue_family.queue_flags.contains(QueueFlags::GRAPHICS)
                && physical_device
                    .surface_support(index as u32, &surface)
                    .unwrap_or(false)
        })
        .expect("could not find a graphics queue family") as u32;
    // println!(
    //     "Device queues: {:?}",
    //     physical_device
    //         .queue_family_properties()
    //         .get(queue_family_index as usize)
    //         .unwrap()
    // );

    match Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
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
