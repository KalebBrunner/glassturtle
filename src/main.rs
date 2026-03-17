// #![allow(unused_imports)]
// #![allow(unused_variables)]

use std::sync::Arc;

mod init_glfw;
mod renderer;
mod setup;
mod shaders;
mod vertex_buffer;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // GLFW stands for Good Luck Fellow Witches and is the window manager
    let (mut glfw, glfw_window, _events) = crate::init_glfw::init_glfw();
    let window = Arc::new(glfw_window);
    let (surface, device) = crate::setup::init_vulkan(window.clone());
    crate::renderer::init_renderer(surface, device);

    while !window.should_close() {
        glfw.poll_events();
    }
}
