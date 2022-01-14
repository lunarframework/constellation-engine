use super::Transform;
use glam::Mat4;

pub struct Camera {
    pub fov: f32,
    pub aspect: f32,
    pub znear: f32,
}

impl Camera {
    pub fn new(fov: f32, aspect: f32, znear: f32) -> Self {
        Self { fov, aspect, znear }
    }

    pub fn compute_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fov, self.aspect, self.znear)
    }

    pub fn compute_view_matrix(&self, transform: &Transform) -> Mat4 {
        transform.compute_matrix().inverse()
    }
}
