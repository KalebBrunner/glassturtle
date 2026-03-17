use std::sync::Arc;

use vulkano::{
    image::{Image, view::ImageView},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

/// This function is called once during initialization, then again whenever the window is resized.
pub fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: [view].to_vec(),
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
