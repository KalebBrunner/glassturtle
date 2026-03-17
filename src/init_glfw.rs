use glfw::{ClientApiHint, Glfw, GlfwReceiver, PWindow, WindowHint, fail_on_errors};

pub fn init_glfw() -> (
    Glfw,
    PWindow,
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
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    // glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::TransparentFramebuffer(true));
    // glfw.window_hint(WindowHint::Decorated(false));
    glfw.window_hint(WindowHint::MousePassthrough(true));

    let (mut window, events) = glfw
        .create_window(640, 480, "Glass Turtle.", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_framebuffer_size_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_pos_polling(true);

    // let ext = get_required_extensions(&glfw);
    (glfw, window, events)
}
