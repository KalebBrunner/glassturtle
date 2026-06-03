use glam::Vec3;
use glfw::{Action, Key, WindowEvent};

use crate::camera::Camera;

const CAMERA_MOVE_SPEED: f32 = 0.1;

const CAMERA_ORBIT_SPEED: f32 = 0.05;

pub fn handle_keybinds(event: &WindowEvent, camera: &mut Camera) {
    let WindowEvent::Key(key, _, action, _) = event else {
        return;
    };

    if !matches!(action, Action::Press | Action::Repeat) {
        return;
    }

    match key {
        Key::Left => camera.orbit_y(CAMERA_ORBIT_SPEED),
        Key::Right => camera.orbit_y(-CAMERA_ORBIT_SPEED),
        Key::Up => camera.orbit_x(CAMERA_ORBIT_SPEED),
        Key::Down => camera.orbit_x(-CAMERA_ORBIT_SPEED),
        _ => {}
    }
}
