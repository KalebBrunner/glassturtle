// #![allow(unused_imports)]
// #![allow(unused_variables)]

mod a_glfw;
mod b_vulkan;
mod c_vertex_buffers;
mod d_render_context;

use crate::a_glfw::init_glfw;
use crate::b_vulkan::init_vulkan;
use crate::c_vertex_buffers::init_vertex_buffer;
use crate::d_render_context::{App, init_render_context};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW stands for Good Luck Fellow Witches and is the window manager
    let (mut glfw, window, events) = init_glfw();
    let (_vulkan, surface, device, queue) = init_vulkan(window.clone());

    let (command_buffer_allocator, vertex_buffer) = init_vertex_buffer(device.clone());

    let render_context = init_render_context(window.clone(), surface, device.clone());

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
