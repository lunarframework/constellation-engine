use crate::{App, AppEvent, AppState, ImageId, RenderHandle};

use crate::universe::{Camera, StarRenderer};

use clap::ArgMatches;

use starlight::prelude::*;

struct RenderArea {
    target: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,

    target_id: Option<ImageId>,
}

impl RenderArea {
    fn new(renderer: &RenderHandle) -> Self {
        let target = renderer.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Area"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            target,
            view,
            width: 1,
            height: 1,

            target_id: None,
        }
    }

    fn resize(&mut self, renderer: &RenderHandle, width: u32, height: u32) -> ImageId {
        if (self.width != width || self.height != height) && (width > 0 && height > 0) {
            self.target = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Render Area"),
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            self.view = self
                .target
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.width = width;
            self.height = height;

            if self.target_id.is_some() {
                renderer.unregister_image(self.target_id.take().unwrap());
            }
        }

        if self.target_id.is_none() {
            self.target_id = Some(renderer.register_image(&self.target, wgpu::FilterMode::Linear));
        }

        self.target_id.unwrap()
    }
}

pub fn open(_matches: &ArgMatches) {
    // Initialize the app
    let app = App::new();

    // Retrieve the app context
    let context = app.context();

    let renderer = context.renderer();

    // Render Area/Viewport
    let mut render_area = RenderArea::new(&renderer);

    let mut camera = Camera::new(1.0, 3.14 / 6.0, 0.0001, 1000.0);

    let mut star_renderer =
        StarRenderer::new(renderer.clone(), wgpu::TextureFormat::Rgba8UnormSrgb);

    // Physics Entities
    let universe = World::new();

    app.run(move |event| match event {
        AppEvent::CloseRequested => AppState::Exit,
        AppEvent::Frame { ctx } => {
            // Fill image with color

            // *********************
            // UI CONSTRUCTION *****
            // *********************

            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.strong("TOOL BAR");
                let point_size = ui.available_size();
                let pixel_size = ui.available_size() * ctx.pixels_per_point();

                camera.set_aspect(pixel_size[0] as f32 / pixel_size[1] as f32);

                // Resize render area
                let id = render_area.resize(&renderer, pixel_size[0] as u32, pixel_size[1] as u32);

                ui.image(id.to_egui(), point_size);
            });

            // **********************
            // MAIN VIEWPORT ********
            // **********************

            star_renderer.prepare(&universe, &camera);

            let mut encoder = context.renderer().device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Command Encoder"),
                },
            );

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &render_area.view,
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

            // star_renderer.render(&mut render_pass);

            drop(render_pass);

            context
                .renderer()
                .queue()
                .submit(std::iter::once(encoder.finish()));

            AppState::Run
        }
    });
}
