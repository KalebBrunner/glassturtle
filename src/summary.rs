use std::sync::Arc;

use vulkano::{
    device::{Device, Queue, QueueFlags},
    instance::Instance,
    swapchain::{CompositeAlpha, CompositeAlphas, PresentMode, Surface, SurfaceInfo},
};

pub fn print_vulkan_project_summary(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device: &Arc<Device>,
    queue: &Arc<Queue>,
) {
    println!();
    println!("================ Vulkan Project Summary ================");

    print_instance_device_summary(instance, surface);
    print_selected_device_summary(device);
    print_selected_queue_summary(device, queue, surface);
    print_swapchain_support_summary(device, surface);

    println!("========================================================");
    println!();
}

fn print_instance_device_summary(instance: &Arc<Instance>, surface: &Arc<Surface>) {
    println!();
    println!("Instance / physical devices:");

    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices");

    for (device_index, physical_device) in physical_devices.enumerate() {
        let properties = physical_device.properties();
        let extensions = physical_device.supported_extensions();

        println!("  [{}] {}", device_index, properties.device_name);
        println!("      type: {:?}", properties.device_type);
        println!("      api version: {:?}", properties.api_version);
        println!("      vendor id: {:?}", properties.vendor_id);
        println!("      device id: {:?}", properties.device_id);
        println!("      supports khr_swapchain: {}", extensions.khr_swapchain);

        println!("      queue families:");

        for (queue_index, queue_family) in
            physical_device.queue_family_properties().iter().enumerate()
        {
            let supports_present = physical_device
                .surface_support(queue_index as u32, surface)
                .unwrap_or(false);

            println!(
                "        [{}] flags: {:?}, count: {}, present: {}",
                queue_index, queue_family.queue_flags, queue_family.queue_count, supports_present,
            );
        }
    }
}

fn print_selected_device_summary(device: &Arc<Device>) {
    let physical_device = device.physical_device();
    let properties = physical_device.properties();

    println!();
    println!("Selected physical device:");
    println!("  name: {}", properties.device_name);
    println!("  type: {:?}", properties.device_type);
    println!("  api version: {:?}", properties.api_version);
    println!("  vendor id: {:?}", properties.vendor_id);
    println!("  device id: {:?}", properties.device_id);

    println!();
    println!("Selected device extension support:");
    println!(
        "  khr_swapchain supported: {}",
        physical_device.supported_extensions().khr_swapchain
    );

    println!();
    println!("Selected device notable supported features:");
    let features = physical_device.supported_features();
    println!("  geometry_shader: {}", features.geometry_shader);
    println!("  tessellation_shader: {}", features.tessellation_shader);
    println!("  sampler_anisotropy: {}", features.sampler_anisotropy);
    println!("  fill_mode_non_solid: {}", features.fill_mode_non_solid);
    println!("  wide_lines: {}", features.wide_lines);
    println!("  multi_draw_indirect: {}", features.multi_draw_indirect);
    println!("  shader_int64: {}", features.shader_int64);
}

fn print_selected_queue_summary(device: &Arc<Device>, queue: &Arc<Queue>, surface: &Arc<Surface>) {
    let physical_device = device.physical_device();
    let queue_family_index = queue.queue_family_index();

    println!();
    println!("Selected queue:");
    println!("  queue family index: {}", queue_family_index);

    let queue_family = physical_device
        .queue_family_properties()
        .get(queue_family_index as usize)
        .expect("selected queue family index was invalid");

    let supports_present = physical_device
        .surface_support(queue_family_index, surface)
        .unwrap_or(false);

    println!("  flags: {:?}", queue_family.queue_flags);
    println!("  queue count in family: {}", queue_family.queue_count);
    println!(
        "  graphics: {}",
        queue_family.queue_flags.contains(QueueFlags::GRAPHICS)
    );
    println!(
        "  compute: {}",
        queue_family.queue_flags.contains(QueueFlags::COMPUTE)
    );
    println!(
        "  transfer: {}",
        queue_family.queue_flags.contains(QueueFlags::TRANSFER)
    );
    println!("  present to surface: {}", supports_present);
    println!(
        "  timestamp valid bits: {:?}",
        queue_family.timestamp_valid_bits
    );
    println!(
        "  min image transfer granularity: {:?}",
        queue_family.min_image_transfer_granularity
    );
}

pub fn print_swapchain_support_summary(device: &Arc<Device>, surface: &Arc<Surface>) {
    let physical_device = device.physical_device();

    let capabilities = physical_device
        .surface_capabilities(surface, SurfaceInfo::default())
        .expect("failed to query surface capabilities");

    let formats = physical_device
        .surface_formats(surface, SurfaceInfo::default())
        .expect("failed to query surface formats");

    let present_modes = physical_device
        .surface_present_modes(surface, SurfaceInfo::default())
        .expect("failed to query surface present modes");

    println!();
    println!("Swapchain support:");

    println!("  image count:");
    println!("    min: {}", capabilities.min_image_count);
    println!(
        "    max: {}",
        match capabilities.max_image_count {
            Some(count) => count.to_string(),
            None => "no explicit maximum".to_string(),
        }
    );

    println!("  extent:");
    println!(
        "    current: {}",
        match capabilities.current_extent {
            Some(extent) => format_extent(extent),
            None => "not fixed by surface; application chooses".to_string(),
        }
    );
    println!(
        "    min:     {}",
        format_extent(capabilities.min_image_extent)
    );
    println!(
        "    max:     {}",
        format_extent(capabilities.max_image_extent)
    );

    println!("  image array layers:");
    println!("    max: {}", capabilities.max_image_array_layers);

    println!("  transforms:");
    println!("    current:   {:?}", capabilities.current_transform);
    println!("    supported: {:?}", capabilities.supported_transforms);

    println!("  composite alpha:");
    println!(
        "    supported: {:?}",
        capabilities.supported_composite_alpha
    );

    println!("  image usage:");
    println!("    supported: {:?}", capabilities.supported_usage_flags);

    println!("  surface formats:");
    for (index, (format, color_space)) in formats.iter().enumerate() {
        println!("    [{}] {:?} / {:?}", index, format, color_space);
    }

    println!("  present modes:");
    for (index, present_mode) in present_modes.iter().enumerate() {
        println!("    [{}] {:?}", index, present_mode);
    }
}

fn format_extent(extent: [u32; 2]) -> String {
    format!("{} x {}", extent[0], extent[1])
}
