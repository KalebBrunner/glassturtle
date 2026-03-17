use std::sync::Arc;

use vulkano::{
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        physical::{self, PhysicalDevice},
    },
    instance::Instance,
};

pub fn init_physical_device(vulkan: &Arc<Instance>) -> Arc<PhysicalDevice> {
    // list_physical_devices(vulkan);
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

pub fn init_logical_device(
    physical_device: Arc<PhysicalDevice>,
) -> (Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>) {
    let features = DeviceFeatures::empty();
    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let device = match Device::new(
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
    };
    device
}

fn list_physical_devices(instance: &Arc<Instance>) {
    let devices = instance.enumerate_physical_devices().unwrap();
    for device in devices {
        println!("GPU Device: {}", device.properties().device_name);
    }
}
