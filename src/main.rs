#![allow(unused_imports)]
#![allow(unused_variables)]
use std::sync::Arc;
use vulkano::device::physical;
use vulkano::image::SampleCount;
use vulkano::image::{Image, view::ImageView};
use vulkano::instance::{InstanceExtensions, InstanceOwned};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::Surface;

mod app;
mod myglfw;
mod rcx;
mod shaders;
mod summary;
mod vertex_buffer;
mod vulkan;

use crate::app::App;
use crate::myglfw::init_glfw;
use crate::rcx::init_rcx;
use crate::summary::print_vulkan_project_summary;
use crate::vertex_buffer::init_vertex_buffer;
use crate::vulkan::{init_device, init_vkinstance};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches
    let (mut glfw, window, events) = init_glfw();
    let windowing_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");
    let vulkan = init_vkinstance(windowing_extensions);
    let surface =
        Surface::from_window(vulkan.clone(), window.clone()).expect("failed to create surface");
    let (device, queue) = init_device(vulkan.clone(), surface.clone());
    print_vulkan_project_summary(&vulkan, &surface, &device, &queue);

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let render_context = init_rcx(
        surface.clone(),
        device.clone(),
        memory_allocator.clone(),
        SampleCount::Sample8,
    );
    let (command_buffer_allocator, vertex_buffer) =
        init_vertex_buffer(device.clone(), memory_allocator.clone());

    let mut myapp = App {
        window,
        device,
        queue,
        command_buffer_allocator,
        vertex_buffer,
        render_context: Some(render_context),
        sample_count_index: 4,
    };

    while !myapp.window.clone().should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(_, _) => {
                    if let Some(rcx) = myapp.render_context.as_mut() {
                        rcx.recreate_swapchain = true;
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::Right, _, glfw::Action::Press, _) => {
                    myapp.cycle_sample_count(1);
                }
                glfw::WindowEvent::Key(glfw::Key::Left, _, glfw::Action::Press, _) => {
                    myapp.cycle_sample_count(-1);
                }
                _ => {}
            }
        }

        myapp.draw_frame();
    }
}
