use crate::app::{App, AppEvent, AppState};
use crate::components::{Star, Transform};
use crate::render::{Camera, ImageId, RenderCtxRef, UniverseRenderer};
use clap::ArgMatches;
use starlight::prelude::*;

struct Viewport {
    camera: Camera,
}

impl Viewport {
    fn new(render: RenderCtxRef) -> Self {
        let mut camera = Camera::new(render.clone(), wgpu::TextureFormat::Rgba8UnormSrgb, 1, 1);
        camera.set_fovy(3.14 / 4.0);
        camera.set_near(0.001);

        Self { camera }
    }

    fn resize(&mut self, width: u32, height: u32) -> ImageId {
        self.camera.resize(width, height);
        self.camera.image()
    }
}

pub fn open(_matches: &ArgMatches) {
    // Initialize the app
    let app = App::new();

    // Retrieve the app context
    let context = app.context();

    let render = context.render();

    // Viewport
    let mut viewport = Viewport::new(render.clone());

    // Physics Entities
    let mut universe = World::new();

    universe.spawn((Transform::from_xyz(0.0, 0.0, 10.0), Star {}));

    let mut universe_renderer =
        UniverseRenderer::new(render.clone(), wgpu::TextureFormat::Rgba8UnormSrgb);

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

                // Resize render area
                let id = viewport.resize(pixel_size[0] as u32, pixel_size[1] as u32);

                ui.image(id.to_egui(), point_size);
            });

            // **********************
            // MAIN VIEWPORT ********
            // **********************

            universe_renderer.render(&universe, &viewport.camera);

            AppState::Run
        }
    });
}
