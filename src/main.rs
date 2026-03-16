// #![allow(unused_imports)]
// #![allow(unused_variables)]
mod device;
mod glfw;
mod render_pipelines;
mod shaders;
mod swapchain;
mod vertex_buffer;
mod vulkan;
mod vulkan_matcher;

use std::{fmt::Debug, sync::Arc};

use vulkano::swapchain::Surface;

use crate::{
    device::{init_logical_device, init_physical_device},
    glfw::init_glfw,
    render_pipelines::init_renderpass,
    swapchain::init_swapchain,
    vulkan::init_vulkan_instance,
};

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
    let required_extensions =
        Surface::required_extensions(&window).expect("Failed to get required extensions");
    let vulkan = init_vulkan_instance(required_extensions);
    let surface =
        Surface::from_window(vulkan.clone(), window.clone()).expect("failed to create surface");
    let physical_device = init_physical_device(&vulkan);
    let (logical_device, mut queues) = init_logical_device(&physical_device);
    let queue = queues.next().unwrap();
    quick_print(
        "Active queue extensions",
        queue.device().enabled_extensions(),
    );

    let (swapchain, images) = init_swapchain(surface, logical_device);
    quick_print("Swapchain info", &swapchain.create_info());

    let render_pass = init_renderpass(logical_device, swapchain);

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }
}

fn main() {
    pollster::block_on(run());
}
pub fn quick_print<T: Debug>(name: &str, value: &T) {
    println!("{}: {:?}", name, value);
}
