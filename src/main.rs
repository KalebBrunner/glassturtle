#![allow(unused_imports)]
#![allow(unused_variables)]
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::image::{Image, view::ImageView};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::Surface;

mod app;
mod camera;
mod diagnostics_print;
mod mesh;
mod render;
mod shaders;
mod vulkan;
mod windower;

use crate::app::App;
use crate::camera::Camera;
use crate::diagnostics_print::print_diagnostics;

use crate::mesh::create_mesh;
use crate::render::create_render_context;
use crate::vulkan::{create_device, create_vulkan_instance};
use crate::windower::create_window;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches
    let (mut glfw, window, events) = create_window();
    let windowing_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");
    let vulkan = create_vulkan_instance(windowing_extensions);
    let surface =
        Surface::from_window(vulkan.clone(), window.clone()).expect("failed to create surface");
    let (device, queue) = create_device(vulkan.clone(), surface.clone());
    print_diagnostics(&vulkan, &surface, &device, &queue);

    let render_context = create_render_context(surface.clone(), device.clone());

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let mesh = create_mesh(device.clone());

    let camera = Camera::new();

    let mut myapp = App {
        window,
        device,
        queue,
        command_buffer_allocator,
        mesh,
        render_context: render_context,
        camera,
    };

    while !myapp.window.clone().should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(_, _) = event {
                myapp.render_context.recreate_swapchain = true;
            }
        }

        myapp.draw_frame();
    }
}
