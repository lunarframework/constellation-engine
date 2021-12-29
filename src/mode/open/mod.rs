use crate::{App, AppEvent, AppState, Framework, Renderer};

use clap::ArgMatches;

struct RenderArea {
    target: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,

    target_id: Option<egui::TextureId>,
}

impl RenderArea {
    fn new(renderer: &Renderer) -> Self {
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

    fn resize(&mut self, renderer: &Renderer, framework: &mut Framework, width: u32, height: u32) {
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
                framework.unregister_texture(self.target_id.take().unwrap());
            }

            self.target_id = None;
        }
    }

    fn register(&mut self, framework: &mut Framework) -> egui::TextureId {
        if self.target_id.is_none() {
            self.target_id =
                Some(framework.register_texture(&self.target, wgpu::FilterMode::Linear));
        }

        self.target_id.unwrap()
    }
}

pub fn open(_matches: &ArgMatches) {
    // Initialize the app
    let app = App::new();

    // Retrieve the app context
    let context = app.context();

    let mut render_area = RenderArea::new(context.renderer());

    app.run(move |event| match event {
        AppEvent::CloseRequested => AppState::Exit,
        AppEvent::Frame { ctx, frame } => {
            // Fill image with color

            // *********************
            // UI CONSTRUCTION *****
            // *********************

            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.strong("TOOL BAR");
                let point_size = ui.available_size();
                let pizel_size = ui.available_size() * ctx.pixels_per_point();
                // Resize render area
                render_area.resize(
                    context.renderer(),
                    frame,
                    pizel_size[0] as u32,
                    pizel_size[1] as u32,
                );

                let id = render_area.register(frame);

                ui.image(id, point_size);
            });

            // **********************
            // MAIN VIEWPORT ********
            // **********************

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

            drop(render_pass);

            context
                .renderer()
                .queue()
                .submit(std::iter::once(encoder.finish()));

            AppState::Run
        }
    });
}
