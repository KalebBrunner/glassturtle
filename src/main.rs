mod glfw;
mod vulkan;
mod vulkan_matcher;

use crate::{
    glfw::init_window,
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
    let (mut glfw, window, _events) = init_window();
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

    list_physical_devices(&instance);
    let physical_device = instance.enumerate_physical_devices().unwrap().nth(0);

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }
}

fn main() {
    pollster::block_on(run());
}
