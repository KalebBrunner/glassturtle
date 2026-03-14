mod init;
mod while_window;

use crate::init::{State, setup};
use crate::while_window::{key_match, update_state};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let (mut glfw, mut window, events) = setup();
    let mut state = State::new(&mut window).await;

    while !state.window.should_close() {
        glfw.poll_events();

        key_match(&mut state, &events);
        update_state(&mut state)
    }
}
