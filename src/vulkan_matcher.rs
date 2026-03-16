use vulkano::instance::InstanceExtensions;

#[rustfmt::skip]
pub fn match_extensions_names(names: Vec<String>) -> InstanceExtensions {
    
    let mut extensions = InstanceExtensions::empty();
    let mut is_undefined_extension = false;
    for name in names {
        match name.as_str() {
            "VK_KHR_android_surface" => extensions.khr_android_surface = true,
            "VK_KHR_device_group_creation" => extensions.khr_device_group_creation = true,
            "VK_KHR_display" => extensions.khr_display = true,
            "VK_KHR_external_fence_capabilities" => extensions.khr_external_fence_capabilities = true,
            "VK_KHR_external_memory_capabilities" => extensions.khr_external_memory_capabilities = true,
            "VK_KHR_external_semaphore_capabilities" => extensions.khr_external_semaphore_capabilities = true,
            "VK_KHR_get_display_properties2" => extensions.khr_get_display_properties2 = true,
            "VK_KHR_get_physical_device_properties2" => extensions.khr_get_physical_device_properties2 = true,
            "VK_KHR_get_surface_capabilities2" => extensions.khr_get_surface_capabilities2 = true,
            "VK_KHR_portability_enumeration" => extensions.khr_portability_enumeration = true,
            "VK_KHR_surface" => extensions.khr_surface = true,
            "VK_KHR_surface_protected_capabilities" => extensions.khr_surface_protected_capabilities = true,
            "VK_KHR_wayland_surface" => extensions.khr_wayland_surface = true,
            "VK_KHR_win32_surface" => extensions.khr_win32_surface = true,
            "VK_KHR_xcb_surface" => extensions.khr_xcb_surface = true,
            "VK_KHR_xlib_surface" => extensions.khr_xlib_surface = true,
            "VK_EXT_acquire_drm_display" => extensions.ext_acquire_drm_display = true,
            "VK_EXT_acquire_xlib_display" => extensions.ext_acquire_xlib_display = true,
            "VK_EXT_debug_report" => extensions.ext_debug_report = true,
            "VK_EXT_debug_utils" => extensions.ext_debug_utils = true,
            "VK_EXT_direct_mode_display" => extensions.ext_direct_mode_display = true,
            "VK_EXT_directfb_surface" => extensions.ext_directfb_surface = true,
            "VK_EXT_display_surface_counter" => extensions.ext_display_surface_counter = true,
            "VK_EXT_headless_surface" => extensions.ext_headless_surface = true,
            "VK_EXT_layer_settings" => extensions.ext_layer_settings = true,
            "VK_EXT_metal_surface" => extensions.ext_metal_surface = true,
            "VK_EXT_surface_maintenance1" => extensions.ext_surface_maintenance1 = true,
            "VK_EXT_swapchain_colorspace" => extensions.ext_swapchain_colorspace = true,
            "VK_EXT_validation_features" => extensions.ext_validation_features = true,
            "VK_EXT_validation_flags" => extensions.ext_validation_flags = true,
            "VK_FUCHSIA_imagepipe_surface" => extensions.fuchsia_imagepipe_surface = true,
            "VK_GGP_stream_descriptor_surface" => extensions.ggp_stream_descriptor_surface = true,
            "VK_GOOGLE_surfaceless_query" => extensions.google_surfaceless_query = true,
            "VK_LUNARG_direct_driver_loading" => extensions.lunarg_direct_driver_loading = true,
            "VK_MVK_ios_surface" => extensions.mvk_ios_surface = true,
            "VK_MVK_macos_surface" => extensions.mvk_macos_surface = true,
            "VK_NN_vi_surface" => extensions.nn_vi_surface = true,
            "VK_NV_external_memory_capabilities" => extensions.nv_external_memory_capabilities = true,
            "VK_QNX_screen_surface" => extensions.qnx_screen_surface = true,

            other => {
                eprintln!("Unknown/unsupported GLFW Vulkan instance extension: {other}");
                is_undefined_extension = true;
            }
        }
    }
    assert!(!is_undefined_extension, "Undefined extension found");

    extensions
}
