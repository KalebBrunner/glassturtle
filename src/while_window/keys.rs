use glfw::Action::Press;
use glfw::Key::Escape;
use glfw::WindowEvent::{FramebufferSize, Key as KeyEnum, Pos};
use glfw::{GlfwReceiver, WindowEvent};

use crate::init::State;

pub fn key_match(state: &mut State<'_>, events: &GlfwReceiver<(f64, WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            KeyEnum(Escape, _, Press, _) => state.window.set_should_close(true),

            Pos(..) => {
                state.update_surface();
                state.resize(state.size);
            }

            FramebufferSize(width, height) => {
                state.update_surface();
                state.resize((width, height));
            }

            _ => {}
        }
    }
}
