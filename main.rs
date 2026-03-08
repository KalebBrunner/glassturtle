use glfw::{
    Action, Context, Key, VidMode, WindowHint, fail_on_errors,
    ffi::{self, GLFW_POSITION_X},
};

pub fn get_screen_mode(glfw: &mut glfw::Glfw) -> VidMode {
    glfw.with_primary_monitor(|_, monitor_opt| {
        let monitor = monitor_opt.expect("no primary monitor");
        monitor.get_video_mode().expect("no video mode")
    })
}

fn main() {
    // activate glfw
    /*
       GLFW (Graphics Library Framework) is a lightweight utility library for
       use with OpenGL, OpenGL ES and Vulkan. It provides programmers with the
       ability to create and manage windows as well as OpenGL and Vulkan contexts,
       as well as handle joystick, keyboard and mouse input.[3]
    */
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    println!("GLFW initiated");

    // window creation hints
    glfw.window_hint(WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(WindowHint::Decorated(true));
    glfw.window_hint(WindowHint::MousePassthrough(true));

    // create window and event handler
    let (mut window, events) = glfw
        .create_window(100, 100, "GlassTurtle", glfw::WindowMode::Windowed)
        .unwrap();

    // set settings
    window.set_key_polling(true);
    window.make_current();

    // set location
    let display = get_screen_mode(&mut glfw);
    println!("Screen rez: {}x{}", display.width, display.height);

    let (posx, posy) = (
        display.width as i32 - window.get_size().0,
        display.height as i32 - window.get_size().1,
    );
    window.set_pos(posx, posy);
    println!("Screen loc: ({}, {})", posx, posy);

    // activate OpenGL
    /*
        OpenGL (Open Graphics Library[4]) is a cross-language, cross-platform
        application programming interface (API) for rendering 2D and 3D vector
        graphics. The API is typically used to interact with a graphics
        processing unit (GPU), to achieve hardware-accelerated rendering.
    */
    gl::load_with(|symbol| {
        window
            .get_proc_address(symbol)
            .map_or(std::ptr::null(), |p| p as *const _)
    });

    // Window event loop
    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true);
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
