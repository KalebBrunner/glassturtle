use glfw::{ClientApiHint, WindowHint, fail_on_errors};

pub fn init_window() -> (
    glfw::Glfw,
    glfw::PWindow,
    glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::TransparentFramebuffer(true));
    // glfw.window_hint(WindowHint::Decorated(false));
    glfw.window_hint(WindowHint::MousePassthrough(true));

    let (mut window, events) = glfw
        .create_window(800, 600, "Glass Turtle.", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_framebuffer_size_polling(true);
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_pos_polling(true);

    (glfw, window, events)
}
