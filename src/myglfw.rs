use std::sync::Arc;

use glfw::{ClientApiHint, Glfw, GlfwReceiver, PWindow, WindowHint, fail_on_errors, log_errors};

pub fn init_glfw() -> (
    Glfw,
    Arc<PWindow>,
    GlfwReceiver<(f64, glfw::WindowEvent)>,
    // InstanceExtensions,
) {
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
    // let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    let mut glfw = glfw::init(glfw::log_errors!()).unwrap();
    // glfw.window_hint(WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi)); //overrides transparency to false

    // glfw.window_hint(WindowHint::Decorated(false));
    glfw.window_hint(WindowHint::MousePassthrough(true));

    let (mut window, events) = glfw
        .create_window(640, 480, "Glass Turtle.", glfw::WindowMode::Windowed)
        .unwrap();
    println!("client api: {:?}", window.get_client_api());
    println!("decorations?: {}", window.is_decorated());
    println!(
        "transparent framebuffer: {}",
        window.is_framebuffer_transparent()
    );
    println!("mouse passthrough?: {}", window.is_mouse_passthrough());
    window.set_framebuffer_size_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_pos_polling(true);

    let arc_window = Arc::new(window);

    (glfw, arc_window, events)
}
