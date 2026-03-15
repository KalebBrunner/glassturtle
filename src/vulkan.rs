use std::sync::Arc;

use vulkano::{
    Version, VulkanLibrary,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
};

pub fn init_vulkan() -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("failed to load Vulkan loader");

    Instance::new(
        library,
        InstanceCreateInfo {
            engine_name: Some("No Engine".into()),
            engine_version: Version::V1_0,
            ..InstanceCreateInfo::application_from_cargo_toml()
        },
    )
    .expect("failed to create Vulkan instance")
}

fn glfw_extensions_to_vulkano(names: &[String]) -> InstanceExtensions {
    let mut extensions = InstanceExtensions::empty();

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
