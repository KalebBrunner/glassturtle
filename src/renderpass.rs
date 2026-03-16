use std::sync::Arc;

use vulkano::{device::Device, render_pass::RenderPass, swapchain::Swapchain};

pub fn init_renderpass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    // The next step is to create a *render pass*, which is an object that describes where the
    // output of the graphics pipeline will go. It describes the layout of the images where the
    // colors, depth and/or stencil information will be written.
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            // `color` is a custom name we give to the first and only attachment.
            color: {
                // `format: <ty>` indicates the type of the format of the image. This has to be
                // one of the types of the `vulkano::format` module (or alternatively one of
                // your structs that implements the `FormatDesc` trait). Here we use the same
                // format as the swapchain.
                format: swapchain.image_format(),
                // `samples: 1` means that we ask the GPU to use one sample to determine the
                // value of each pixel in the color attachment. We could use a larger value
                // (multisampling) for antialiasing. An example of this can be found in
                // msaa-renderpass.rs.
                samples: 1,
                // `load_op: Clear` means that we ask the GPU to clear the content of this
                // attachment at the start of the drawing.
                load_op: Clear,
                // `store_op: Store` means that we ask the GPU to store the output of the draw
                // in the actual image. We could also ask it to discard the result.
                store_op: Store,
            },
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [color],
            // No depth-stencil attachment is indicated with empty brackets.
            depth_stencil: {},
        },
    )
    .unwrap();

    render_pass
}
