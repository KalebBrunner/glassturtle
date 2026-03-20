// #![allow(unused_imports)]
// #![allow(unused_variables)]
use std::sync::Arc;
use vulkano::image::{Image, view::ImageView};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::Surface;

mod app;
mod myglfw;
mod rcx;
mod shaders;
mod vertex_buffer;
mod vulkan;

use crate::app::App;
use crate::myglfw::init_glfw;
use crate::rcx::init_rcx;
use crate::vertex_buffer::init_vertex_buffer;
use crate::vulkan::{init_device, init_surface, init_vkinstance, init_vulkan};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches
    let (mut glfw, window, events) = init_glfw();
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");
    let vulkan = init_vkinstance(required_extensions);
    let (device, queue) = init_device(vulkan.clone());
    let surface = init_surface(vulkan.clone(), window.clone());

    let (command_buffer_allocator, vertex_buffer) = init_vertex_buffer(device.clone());
    let render_context = init_rcx(window.clone(), surface, device.clone());

    let mut myapp = App {
        device,
        queue,
        command_buffer_allocator,
        vertex_buffer,
        render_context: Some(render_context),
    };

    while !window.clone().should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(_, _) = event
                && let Some(rcx) = myapp.render_context.as_mut()
            {
                rcx.recreate_swapchain = true;
            }
        }

        myapp.draw_frame();
    }
}

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
