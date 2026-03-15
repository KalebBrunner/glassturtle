use std::sync::Arc;

use vulkano::{
    Version, VulkanLibrary,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
};

pub fn init_vulkan(glfw_required_extensions: Vec<String>) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("failed to load Vulkan loader");
    println!("highest Vulkan V: {:?}", library.api_version());

    let extensions = glfw_extensions_to_vulkano(&glfw_required_extensions);

    let create_info = InstanceCreateInfo {
        engine_name: Some("Glass Turtle Graphics".into()),
        engine_version: Version::V1_0,
        max_api_version: Some(Version::V1_0),
        enabled_extensions: extensions,
        ..InstanceCreateInfo::application_from_cargo_toml()
    };

    Instance::new(library, create_info).expect("failed to create Vulkan instance")
}

fn glfw_extensions_to_vulkano(names: &[String]) -> InstanceExtensions {
    let mut extensions: InstanceExtensions = InstanceExtensions::empty();

    for name in names {
        match name.as_str() {
            "VK_KHR_surface" => extensions.khr_surface = true,
            "VK_KHR_wayland_surface" => extensions.khr_wayland_surface = true,
            "VK_KHR_xcb_surface" => extensions.khr_xcb_surface = true,
            "VK_KHR_xlib_surface" => extensions.khr_xlib_surface = true,
            "VK_KHR_win32_surface" => extensions.khr_win32_surface = true,
            "VK_EXT_metal_surface" => extensions.ext_metal_surface = true,
            "VK_MVK_macos_surface" => extensions.mvk_macos_surface = true,
            other => panic!("unhandled GLFW-required instance extension: {other}"),
        }
    }

    extensions
}

pub fn list_physical_devices(instance: Arc<Instance>) {
    for physical_device in instance.enumerate_physical_devices().unwrap() {
        println!("GPU Device: {}", physical_device.properties().device_name);
    }
}
