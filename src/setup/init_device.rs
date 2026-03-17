use std::sync::Arc;
use vulkano::device::{
    DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
    physical::PhysicalDevice,
};
use vulkano::{device::Device, instance::Instance};

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
