use glfw::{Action, Context, Key, WindowHint, fail_on_errors};

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(WindowHint::Decorated(true));
    glfw.window_hint(WindowHint::MousePassthrough(true));

    let (mut window, events) = glfw
        .create_window(800, 300, "Overlay", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|symbol| {
        window
            .get_proc_address(symbol)
            .map_or(std::ptr::null(), |p| p as *const _)
    });

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {}
            }
        }

        let (width, height) = window.get_framebuffer_size();

        unsafe {
            gl::Viewport(0, 0, width, height);

            // transparent background
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // opaque red square
            let square_size = (width.min(height) / 2).max(1);
            let x = (width - square_size) / 2;
            let y = (height - square_size) / 2;

            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(x, y, square_size, square_size);
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::SCISSOR_TEST);
        }

        window.swap_buffers();
    }
}
