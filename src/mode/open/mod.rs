use crate::app::{App, AppEvent, AppState};
use crate::components::{Star, Transform};
use crate::project::Project;
use crate::render::{Camera, ImageId, RenderCtxRef, UniverseRenderer};
use clap::ArgMatches;
use glam::{Quat, Vec3};
use starlight::prelude::*;
use std::path::PathBuf;
use std::process::exit;

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

pub fn open(matches: &ArgMatches) {
    let relative_path = PathBuf::from(
        matches
            .value_of("path")
            .expect("Failed to parse relative path"),
    );

    let working_directory = std::env::current_dir().expect("Failed to find working directory");

    let project_directory = match working_directory.join(relative_path).canonicalize() {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("Failed to canonicalize path");
            eprintln!("{}", error);

            match error.raw_os_error() {
                Some(code) => exit(code),
                None => exit(1),
            }
        }
    };

    let project = Project::load(project_directory);

    // Initialize the app
    let app = App::new();

    // Retrieve the app context
    let context = app.context();

    context
        .window()
        .set_title(format!("Constellation Engine - {}", project.config.name).as_str());

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
                // ui.menu_button("File", |_ui| {});
                // ui.menu_button("Windows", |_ui| {});
                // ui.menu_button("About", |_ui| {});
                ui.horizontal(|ui| {
                    if ui.button("🏠").clicked() {}
                    if ui.button("🔍").clicked() {}
                    if ui.button("↔").clicked() {}
                });
                ui.separator();
                let point_size = ui.available_size();
                let pixel_size = ui.available_size() * ctx.pixels_per_point();

                // Resize render area
                let id = viewport.resize(pixel_size[0] as u32, pixel_size[1] as u32);

                ui.image(id.to_egui(), point_size);
            });

            // **********************
            // Camera Controller ****
            // **********************

            let input = ctx.input();

            const MOVE_SPEED: f32 = 1.0;
            const ROT_ANGLE: f32 = 3.141592;
            let delta_time = input.unstable_dt;

            if input.key_down(egui::Key::W) {
                let translate = Vec3::Z * MOVE_SPEED * delta_time;
                let local = viewport.camera.rotation().mul_vec3(translate);
                viewport.camera.translate(local);
            }

            if input.key_down(egui::Key::S) {
                let translate = -Vec3::Z * MOVE_SPEED * delta_time;
                let local = viewport.camera.rotation().mul_vec3(translate);
                viewport.camera.translate(local);
            }

            if input.key_down(egui::Key::A) {
                let translate = -Vec3::X * MOVE_SPEED * delta_time;
                let local = viewport.camera.rotation().mul_vec3(translate);
                viewport.camera.translate(local);
            }

            if input.key_down(egui::Key::D) {
                let translate = Vec3::X * MOVE_SPEED * delta_time;
                let local = viewport.camera.rotation().mul_vec3(translate);
                viewport.camera.translate(local);
            }

            if input.key_down(egui::Key::Q) {
                let rotation = Quat::from_rotation_z(1.0 / 2.0 * -ROT_ANGLE * delta_time);
                viewport.camera.rotate(rotation);
            }

            if input.key_down(egui::Key::E) {
                let rotation = Quat::from_rotation_z(1.0 / 2.0 * ROT_ANGLE * delta_time);
                viewport.camera.rotate(rotation);
            }

            if input.pointer.button_down(egui::PointerButton::Primary) {
                let xrot = Quat::from_rotation_x(
                    1.0 / 10.0 * ROT_ANGLE * delta_time * input.pointer.delta().y,
                );
                let yrot = Quat::from_rotation_y(
                    1.0 / 10.0 * ROT_ANGLE * delta_time * input.pointer.delta().x,
                );

                viewport.camera.rotate(xrot);
                viewport.camera.rotate(yrot);
            }

            // **********************
            // MAIN VIEWPORT ********
            // **********************

            universe_renderer.render(&universe, &viewport.camera);

            AppState::Run
        }
    });
}
