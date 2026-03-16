#![allow(unused_imports)]
#![allow(unused_variables)]
mod glfw;
mod vulkan;
mod vulkan_matcher;

use std::sync::Arc;

use vulkano::swapchain::Surface;

use crate::{glfw::init_glfw, vulkan::init_vulkan};

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
    let (mut glfw, glfw_window, glfw_events, required_extensions) = init_glfw();
    let window = Arc::new(glfw_window);
    let vulkan = init_vulkan(required_extensions);
    let surface = Surface::from_window(vulkan, window.clone()).expect("failed to create surface");

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }
}

fn main() {
    pollster::block_on(run());
}
