mod glfw;
use crate::glfw::initWindow;

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
    let (mut glfw, mut window, events) = initWindow();
    print!("{:?}", window.get_context_version());

    // get custom state object
    // let mut state = State::new(&mut window).await;

    while !window.should_close() {
        glfw.poll_events();

        // key_match(&mut state, &events);
        // update_state(&mut state)
    }
}

fn main() {
    pollster::block_on(run());
}
