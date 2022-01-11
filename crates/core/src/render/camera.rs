use super::{ImageId, RenderCtxRef};
use glam::{Mat4, Quat, Vec3, Vec4};

/// Allows 3D data to be rendered to a given 2D target.
pub struct Camera {
    render: RenderCtxRef,

    target: wgpu::Texture,
    view: wgpu::TextureView,
    image: ImageId,

    width: u32,
    height: u32,

    position: Vec3,
    rotation: Quat,
    scale: Vec3,

    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(render: RenderCtxRef, width: u32, height: u32) -> Self {
        let target = render.create_ldr_attachment(width, height, 1, true);
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let image = render.register_image(&target, wgpu::FilterMode::Linear);

        Self {
            render,
            target,
            view,
            image,
            width,
            height,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,

            fovy: 3.14 / 4.0,
            near: 0.01,
            far: 1000.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if (self.width != width || self.height != height) && (width > 0 && height > 0) {
            self.width = width;
            self.height = height;

            self.target = self.render.create_ldr_attachment(width, height, 1, true);
            self.view = self
                .target
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.render.unregister_image(self.image);

            self.image = self
                .render
                .register_image(&self.target, wgpu::FilterMode::Linear);
        }
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn near(&self) -> f32 {
        self.near
    }

    pub fn far(&self) -> f32 {
        self.far
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    pub fn target(&self) -> &wgpu::Texture {
        &self.target
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn image(&self) -> ImageId {
        self.image
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Vec3 {
        &mut self.position
    }

    pub fn rotation(&self) -> &Quat {
        &self.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut Quat {
        &mut self.rotation
    }

    pub fn scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn scale_mut(&mut self) -> &mut Vec3 {
        &mut self.scale
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation *= rotation;
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    pub fn compute_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fovy, self.aspect(), self.near)
        // Mat4::perspective_lh(self.fovy, self.aspect(), self.near, self.far)
    }

    pub fn compute_view_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position).inverse()
    }

    pub fn compute_proj_view_matrix(&self) -> Mat4 {
        self.compute_projection_matrix() * self.compute_view_matrix()
    }

    /// Returns the amount (on each axis) a length will be scale at a certain z value.
    pub fn scale_factor(&self, distance: f32) -> f32 {
        1.0 / (self.compute_projection_matrix() * Vec4::new(0.0, 0.0, distance, 1.0)).w
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.render.unregister_image(self.image);
    }
}
