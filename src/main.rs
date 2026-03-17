// #![allow(unused_imports)]
// #![allow(unused_variables)]

mod device;
mod frame_buffer;
mod glfw;
mod pipeline;
mod renderpass;
mod shaders;
mod swapchain;
mod vertex_buffer;
mod vulkan;
mod vulkan_matcher;

use std::{fmt::Debug, sync::Arc};

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, swapchain::Surface,
};

use crate::{glfw::init_glfw, vulkan::init_vulkan};

#[derive(BufferContents, Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct MyTriangleVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

async fn run() {
    /*
    https://en.wikipedia.org/wiki/GLFW
    GLFW (Good Luck Future Witches) is a lightweight utility library for
    use with OpenGL, OpenGL ES and Vulkan. It provides programmers with
    the ability to create and manage
    - windows
    - OpenGL contexts
    - Vulkan contexts
    as well as handle
    - joysticks
    - keyboards
    - mice
     */
    let (mut glfw, glfw_window, glfw_events) = init_glfw();
    let window = Arc::new(glfw_window);
    init_vulkan(window.clone());

    while !window.should_close() {
        glfw.poll_events();

        //     for (_, event) in glfw::flush_messages(&events) {
        //         match event {
        //             WindowEvent::FramebufferSize(_, _) => {
        //                 renderer.recreate_swapchain = true;
        //             }
        //             WindowEvent::Close => {
        //                 // if needed
        //             }
        //             WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
        //                 // if you kept window mutable instead of Arc-only access:
        //                 // glfw_window.set_should_close(true);
        //             }
        //             _ => {}
        //         }
    }

    //     draw_frame(&mut renderer, &window);
    // }

    // wait_idle(&renderer);
}

fn main() {
    pollster::block_on(run());
}
pub fn quick_print<T: Debug>(name: &str, value: &T) {
    println!("{}: {:?}", name, value);
}

// other created functions
// window_size_dependent_setup
// init_vertex_bufffer
// fs and vs shaders
