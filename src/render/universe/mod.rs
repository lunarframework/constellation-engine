use starlight::World;

use super::{Camera, RenderCtxRef};

mod star;

use star::StarPipeline;

pub struct UniverseRenderer {
    render: RenderCtxRef,
    star_pipeline: StarPipeline,
}

impl UniverseRenderer {
    pub fn new(render: RenderCtxRef, output_format: wgpu::TextureFormat) -> Self {
        let star_pipeline = StarPipeline::new(render.clone(), output_format);
        Self {
            render,
            star_pipeline,
        }
    }

    pub fn render(&mut self, world: &World, camera: &Camera) {
        self.star_pipeline.update(world, camera);

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
            depth_stencil_attachment: None,
        });

        self.star_pipeline.render(&mut render_pass, camera);

        drop(render_pass);

        self.render
            .queue()
            .submit(std::iter::once(encoder.finish()));
    }
}
