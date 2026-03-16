#![allow(unused_imports)]
#![allow(unused_variables)]
mod glfw;
mod surface;
mod vulkan;
mod vulkan_matcher;

use std::sync::Arc;

use crate::{
    glfw::init_window,
    surface::create_surface,
    vulkan::{init_vulkan, list_physical_devices},
    vulkan_matcher::match_extensions_names,
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
    let (mut glfw, glfw_window, _events) = init_window();
    let window = Arc::new(glfw_window);
    println!("GLFW context: {:?}", glfw.get_platform());

    let glfw_required_extensions = glfw
        .get_required_instance_extensions()
        .expect("GLFW did not return Vulkan instance extensions; Vulkan may be unavailable");
    println!(
        "Extensions required by glfw = {:?}",
        glfw_required_extensions
    );

    let required_extensions = match_extensions_names(glfw_required_extensions);

    let instance = init_vulkan(required_extensions);
    println!("Vulkan instance api: {:?}", instance.api_version());

    let surface = create_surface(instance.clone(), window.clone());

    list_physical_devices(&instance);
    let device_id = 0;
    let physical_device = instance
        .enumerate_physical_devices()
        .unwrap()
        .nth(device_id)
        .expect("Selected Vulkan physical device not found");

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }

    let queue_family = physical_device.queue_family_properties();
    println!("{:?}", queue_family);
}

fn main() {
    pollster::block_on(run());
}
