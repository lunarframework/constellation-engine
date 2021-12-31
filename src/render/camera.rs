use super::{ImageId, RenderCtxRef};
use glam::{Mat4, Quat, Vec3};

/// Allows 3D data to be rendered to a given 2D target.
pub struct Camera {
    render: RenderCtxRef,

    target: wgpu::Texture,
    view: wgpu::TextureView,
    image: ImageId,
    format: wgpu::TextureFormat,

    width: u32,
    height: u32,

    position: Vec3,
    rotation: Quat,
    scale: Vec3,

    fovy: f32,
    near: f32,
}

impl Camera {
    pub fn new(render: RenderCtxRef, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let target = render.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Camera Render Texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            sample_count: 1,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let image = render.register_image(&target, wgpu::FilterMode::Linear);

        Self {
            render,
            target,
            view,
            image,
            format,
            width,
            height,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,

            fovy: 3.14 / 4.0,
            near: 0.01,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if (self.width != width || self.height != height) && (width > 0 && height > 0) {
            self.width = width;
            self.height = height;
            self.target = self
                .render
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Camera Render Texture"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    sample_count: 1,
                    format: self.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                });

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

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
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

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
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

    pub fn compute_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_infinite_reverse_lh(self.fovy, self.aspect(), self.near)
    }

    pub fn compute_view_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position).inverse()
    }

    pub fn compute_proj_view_matrix(&self) -> Mat4 {
        self.compute_projection_matrix() * self.compute_view_matrix()
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.render.unregister_image(self.image);
    }
}
