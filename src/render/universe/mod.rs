use starlight::World;

use super::{Camera, RenderCtxRef};

mod star;

use star::StarPipeline;

pub struct UniverseRenderer {
    render: RenderCtxRef,
    star_pipeline: StarPipeline,

    depth_target: wgpu::Texture,
    depth_view: wgpu::TextureView,
    depth_width: u32,
    depth_height: u32,
}

impl UniverseRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    pub fn new(render: RenderCtxRef, output_format: wgpu::TextureFormat) -> Self {
        let star_pipeline = StarPipeline::new(render.clone(), output_format, Self::DEPTH_FORMAT);

        let depth_target = render.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Universe Depth Texture"),
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let depth_view = depth_target.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            render,
            star_pipeline,
            depth_target,
            depth_view,
            depth_width: 1,
            depth_height: 1,
        }
    }

    pub fn render(&mut self, world: &World, camera: &Camera, dt: f32) {
        self.star_pipeline.update(world, camera, dt);

        if camera.width() != self.depth_width || camera.height() != self.depth_height {
            self.depth_width = camera.width();
            self.depth_height = camera.height();
            self.depth_target = self
                .render
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Universe Depth Texture"),
                    dimension: wgpu::TextureDimension::D2,
                    format: Self::DEPTH_FORMAT,
                    mip_level_count: 1,
                    sample_count: 1,
                    size: wgpu::Extent3d {
                        width: self.depth_width,
                        height: self.depth_height,
                        depth_or_array_layers: 1,
                    },
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                });

            self.depth_view = self
                .depth_target
                .create_view(&wgpu::TextureViewDescriptor::default());
        }

        let mut encoder =
            self.render
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Universe Command Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Universe Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: camera.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        self.star_pipeline.render(&mut render_pass, camera);

        drop(render_pass);

        self.render
            .queue()
            .submit(std::iter::once(encoder.finish()));
    }
}
