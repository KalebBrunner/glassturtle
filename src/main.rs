// #![allow(unused_imports)]
// #![allow(unused_variables)]

mod d_render_context;
mod myglfw;
mod struct_my_vertex;
mod vertex_buffer;
mod vulkan;

use crate::d_render_context::App;
use crate::d_render_context::init_render_context;
use crate::myglfw::init_glfw;
use crate::vertex_buffer::init_vertex_buffer;
use crate::vulkan::init_vulkan;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW is the window manager api. It stands for Good Luck Fellow Witches
    let (mut glfw, window, events) = init_glfw();
    // Vulkan is the api that talks to the graphics card
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
