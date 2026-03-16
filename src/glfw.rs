use std::sync::Arc;

use glfw::{ClientApiHint, Glfw, GlfwReceiver, PWindow, Window, WindowHint, fail_on_errors};
use vulkano::instance::InstanceExtensions;

use crate::vulkan_matcher::match_extensions_names;

pub fn init_glfw() -> (
    Glfw,
    PWindow,
    GlfwReceiver<(f64, glfw::WindowEvent)>,
    // InstanceExtensions,
) {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    glfw.window_hint(WindowHint::Resizable(false));
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

fn get_required_extensions(glfw: &Glfw) -> InstanceExtensions {
    let glfw_required_extensions = glfw
        .get_required_instance_extensions()
        .expect("GLFW did not return Vulkan instance extensions; Vulkan may be unavailable");
    println!(
        "Extensions required by glfw = {:?}",
        glfw_required_extensions
    );

    match_extensions_names(glfw_required_extensions)
}
