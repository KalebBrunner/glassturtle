mod glfw;
mod vulkan;
use crate::{glfw::init_window, vulkan::init_vulkan};

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
    let (mut glfw, mut window, events) = init_window();
    println!("GLFW context: {:?}", glfw.get_platform());

    let instance = init_vulkan();
    println!("Vulkan api: {:?}", instance.api_version());

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }
}

fn main() {
    pollster::block_on(run());
}
