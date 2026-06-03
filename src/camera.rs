use glam::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy_radians: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: Vec3::new(2.0, 2.0, 2.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fovy_radians: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn world_to_clip(&self, aspect: f32) -> Mat4 {
        let projection = Mat4::perspective_rh_gl(self.fovy_radians, aspect, self.near, self.far);

        let view = Mat4::look_at_rh(self.eye, self.target, self.up);

        projection * view
    }
}
