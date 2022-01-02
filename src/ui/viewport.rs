use crate::render::{Camera, ImageId, RenderCtxRef};
use egui::Ui;
use glam::{Quat, Vec3};

/// Represents a viewport into a world. This Ui element manifests
/// as an image that takes up the remaining space in a frame.
/// It holds a camera into the three d world, and allows for
/// different renderers to render to the rsulting image.
pub struct Viewport {
    camera: Camera,
}

impl Viewport {
    pub fn new(render: RenderCtxRef) -> Self {
        let mut camera = Camera::new(render.clone(), wgpu::TextureFormat::Rgba8UnormSrgb, 1, 1);
        camera.set_fovy(3.14 / 4.0);
        camera.set_near(0.001);

        Self { camera }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn resize(&mut self, width: u32, height: u32) -> ImageId {
        self.camera.resize(width, height);
        self.camera.image()
    }
}

impl egui::Widget for &mut Viewport {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let desired_size = ui.available_size();

        let (rect, response) = ui.allocate_exact_size(
            desired_size,
            egui::Sense {
                click: true,
                drag: true,
                focusable: true,
            },
        );

        if response.is_pointer_button_down_on() {
            response.request_focus();
        }

        if ui.is_rect_visible(rect) {
            let size = rect.size() * ui.ctx().pixels_per_point();
            let id = self.resize(size[0] as u32, size[1] as u32);
            // Render image
            let mut mesh = epaint::Mesh::with_texture(id.to_egui());
            let uv = epaint::Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0);
            let tint = epaint::Color32::WHITE;
            mesh.add_rect_with_uv(rect, uv, tint);
            ui.painter().add(egui::Shape::Mesh(mesh));
        }

        // **********************
        // Camera Controller ****
        // **********************
        let input = ui.input();

        const MOVE_SPEED: f32 = 1.0;
        const ROT_ANGLE: f32 = 3.141592;
        let delta_time = input.unstable_dt;

        if response.has_focus() {
            if input.key_down(egui::Key::W) {
                let translate = Vec3::Z * MOVE_SPEED * delta_time;
                let local = self.camera.rotation().mul_vec3(translate);
                self.camera.translate(local);
            }

            if input.key_down(egui::Key::S) {
                let translate = -Vec3::Z * MOVE_SPEED * delta_time;
                let local = self.camera.rotation().mul_vec3(translate);
                self.camera.translate(local);
            }

            if input.key_down(egui::Key::A) {
                let translate = -Vec3::X * MOVE_SPEED * delta_time;
                let local = self.camera.rotation().mul_vec3(translate);
                self.camera.translate(local);
            }

            if input.key_down(egui::Key::D) {
                let translate = Vec3::X * MOVE_SPEED * delta_time;
                let local = self.camera.rotation().mul_vec3(translate);
                self.camera.translate(local);
            }

            if input.key_down(egui::Key::Q) {
                let rotation = Quat::from_rotation_z(1.0 / 2.0 * ROT_ANGLE * delta_time);
                self.camera.rotate(rotation);
            }

            if input.key_down(egui::Key::E) {
                let rotation = Quat::from_rotation_z(1.0 / 2.0 * -ROT_ANGLE * delta_time);
                self.camera.rotate(rotation);
            }
        }

        if response.dragged() {
            let delta = response.drag_delta();
            let xrot = Quat::from_rotation_x(1.0 / 10.0 * ROT_ANGLE * delta_time * delta.y);
            let yrot = Quat::from_rotation_y(1.0 / 10.0 * ROT_ANGLE * delta_time * delta.x);

            self.camera.rotate(xrot);
            self.camera.rotate(yrot);
        }

        response
    }
}
