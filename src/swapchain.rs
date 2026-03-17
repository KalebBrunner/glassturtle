use std::{
    cmp::{max, min},
    sync::Arc,
};

use vulkano::{
    device::Device,
    image::{Image, ImageUsage},
    swapchain::{
        CompositeAlpha, FullScreenExclusive, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
    },
};

pub fn init_swapchain(
    surface: Arc<Surface>,
    logical_device: &Arc<Device>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = logical_device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    // Use the current window size or some fixed resolution.
    let image_extent = surface_capabilities.current_extent.unwrap_or([640, 480]);

    // Try to use double-buffering.
    let min_image_count = match surface_capabilities.max_image_count {
        None => max(2, surface_capabilities.min_image_count),
        Some(limit) => min(max(2, surface_capabilities.min_image_count), limit),
    };

    // Preserve the current surface transform.
    let pre_transform = surface_capabilities.current_transform;

    // Use the first available format.
    let (image_format, color_space) = logical_device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .unwrap()[0];
    let composite_alpha = CompositeAlpha::Inherit;
    let present_mode = PresentMode::Fifo;
    let full_screen_exclusive = FullScreenExclusive::Default;

    let swap_info = SwapchainCreateInfo {
        // How many images to use in the swapchain.
        min_image_count,
        // The format of the images.
        image_format,
        // The size of each image.
        image_extent,
        // The created swapchain images will be used as a color attachment for rendering.
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        // What transformation to use with the surface.
        pre_transform,
        // How to handle the alpha channel.
        composite_alpha,
        // How to present images.
        present_mode,
        // How to handle full-screen exclusivity
        full_screen_exclusive,
        ..Default::default()
    };

    // Create the swapchain and its images.
    let (swapchain, images) = Swapchain::new(
        // Create the swapchain in this `device`'s memory.
        logical_device.clone(), // The surface where the images will be presented.
        surface,                // The creation parameters.
        swap_info,
    )
    .unwrap();

    (swapchain, images)
}
