use crate::app::{App, AppEvent, AppState};
use crate::components::{Star, Transform};
use crate::project::Project;
use crate::render::{BloomSettings, RendererSettings, StarSettings, UniverseRenderer};
use crate::ui::Viewport;
use clap::ArgMatches;
use glam::Vec4;
use starlight::prelude::*;
use std::path::PathBuf;
use std::process::exit;

fn star_editor(ctx: &egui::CtxRef, star: &mut Star) {
    egui::Window::new("Star Editor")
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label("Radius");
            ui.add(egui::DragValue::new(&mut star.radius).clamp_range(0.0f32..=10.0));
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong("Granules");
                    ui.label("Octaves");
                    ui.add(
                        egui::DragValue::new(&mut star.granule_octaves).clamp_range(0.0f32..=10.0),
                    );
                    ui.label("Lacunarity");
                    ui.add(
                        egui::DragValue::new(&mut star.granule_lacunarity)
                            .clamp_range(0.0f32..=1000.0),
                    );
                    ui.label("Gain");
                    ui.add(
                        egui::DragValue::new(&mut star.granule_gain)
                            .clamp_range(0.0f32..=1.0)
                            .speed(0.1),
                    );
                });

                ui.vertical(|ui| {
                    ui.strong("Sunspots");
                    ui.label("Sharpness");
                    ui.add(
                        egui::DragValue::new(&mut star.sunspot_sharpness)
                            .clamp_range(0.0f32..=1000.0),
                    );
                    ui.label("Cutoff");
                    ui.add(
                        egui::DragValue::new(&mut star.sunspots_cutoff).clamp_range(0.0f32..=1.0),
                    );
                    ui.label("Frequency");
                    ui.add(
                        egui::DragValue::new(&mut star.sunspots_frequency)
                            .clamp_range(0.0f32..=1.0)
                            .speed(0.001),
                    );
                    ui.label("Radius");
                });
            });

            ui.horizontal(|ui| {
                let mut color = egui::Color32::from_rgba_premultiplied(
                    (255.0 * star.color[0]) as u8,
                    (255.0 * star.color[1]) as u8,
                    (255.0 * star.color[2]) as u8,
                    (255.0 * star.color[3]) as u8,
                );

                ui.strong("Color");
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                );

                star.color[0] = color[0] as f32 / 255 as f32;
                star.color[1] = color[1] as f32 / 255 as f32;
                star.color[2] = color[2] as f32 / 255 as f32;
                star.color[3] = color[3] as f32 / 255 as f32;

                let mut shifted_color = egui::Color32::from_rgba_premultiplied(
                    (255.0 * star.shift[0]) as u8,
                    (255.0 * star.shift[1]) as u8,
                    (255.0 * star.shift[2]) as u8,
                    (255.0 * star.shift[3]) as u8,
                );

                ui.strong("Shift");
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut shifted_color,
                    egui::color_picker::Alpha::Opaque,
                );

                star.shift[0] = shifted_color[0] as f32 / 255 as f32;
                star.shift[1] = shifted_color[1] as f32 / 255 as f32;
                star.shift[2] = shifted_color[2] as f32 / 255 as f32;
                star.shift[3] = shifted_color[3] as f32 / 255 as f32;
            });
        });
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

    let _star = universe.spawn((
        Transform::from_xyz(0.0, 0.0, 10.0),
        Star {
            radius: 1.0,
            granule_lacunarity: 40.0,
            granule_gain: 0.5,
            granule_octaves: 3.0,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            shift: Vec4::new(0.0, 0.0, 0.0, 1.0),
            sunspot_sharpness: 2.0,
            sunspots_cutoff: 0.3,
            sunspots_frequency: 0.001,
        },
    ));

    universe.spawn((
        Transform::from_xyz(0.0, 0.0, 20.0),
        Star {
            radius: 1.0,
            granule_lacunarity: 40.0,
            granule_gain: 0.5,
            granule_octaves: 3.0,
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            shift: Vec4::new(0.0, 0.0, 0.0, 1.0),
            sunspot_sharpness: 2.0,
            sunspots_cutoff: 0.3,
            sunspots_frequency: 0.001,
        },
    ));

    universe.spawn((
        Transform::from_xyz(0.0, 0.0, 30.0),
        Star {
            radius: 1.0,
            granule_lacunarity: 40.0,
            granule_gain: 0.5,
            granule_octaves: 3.0,
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            shift: Vec4::new(0.0, 0.0, 0.0, 1.0),
            sunspot_sharpness: 2.0,
            sunspots_cutoff: 0.3,
            sunspots_frequency: 0.001,
        },
    ));

    let mut renderer_settings = RendererSettings {
        bloom: BloomSettings {
            threshold: 0.4,
            knee: 0.3,
            intensity: 1.0,
        },
        star: StarSettings {
            anim_time: 0.0,
            min_size_for_rays: 0.01,
        },
        exposure: 1.0,
    };
    let mut universe_renderer = UniverseRenderer::new(render.clone());

    app.run(move |event| match event {
        AppEvent::CloseRequested => AppState::Exit,
        AppEvent::Frame { ctx } => {
            // Fill image with color

            // *********************
            // UI CONSTRUCTION *****
            // *********************

            egui::Window::new("Star Editor")
                .collapsible(true)
                .show(&ctx, |ui| {
                    ui.label("Threshold");
                    ui.add(egui::DragValue::new(&mut renderer_settings.bloom.threshold));
                    ui.label("Knee");
                    ui.add(egui::DragValue::new(&mut renderer_settings.bloom.knee));
                    ui.label("Intensity");
                    ui.add(egui::DragValue::new(&mut renderer_settings.bloom.intensity));

                    ui.label("Exposure");
                    ui.add(egui::DragValue::new(&mut renderer_settings.exposure));
                });

            egui::CentralPanel::default().show(&ctx, |ui| {
                // ui.menu_button("File", |_ui| {});
                // ui.menu_button("Windows", |_ui| {});
                // ui.menu_button("About", |_ui| {});
                // ui.horizontal(|ui| {
                //     if ui.button("🏠").clicked() {}
                //     if ui.button("🔍").clicked() {}
                //     if ui.button("↔").clicked() {}
                // });
                // ui.separator();

                ui.horizontal(|ui| {
                    // ui.vertical(|ui| {
                    //     let star = &mut universe.get_mut::<Star>(star).unwrap();

                    //     ui.label("Radius");
                    //     ui.add(egui::DragValue::new(&mut star.radius).clamp_range(0.0f32..=10.0));
                    //     ui.horizontal(|ui| {
                    //         ui.vertical(|ui| {
                    //             ui.strong("Granules");
                    //             ui.label("Octaves");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.granule_octaves)
                    //                     .clamp_range(0.0f32..=10.0),
                    //             );
                    //             ui.label("Lacunarity");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.granule_lacunarity)
                    //                     .clamp_range(0.0f32..=1000.0),
                    //             );
                    //             ui.label("Gain");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.granule_gain)
                    //                     .clamp_range(0.0f32..=1.0)
                    //                     .speed(0.1),
                    //             );
                    //         });

                    //         ui.vertical(|ui| {
                    //             ui.strong("Sunspots");
                    //             ui.label("Sharpness");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.sunspot_sharpness)
                    //                     .clamp_range(0.0f32..=1000.0),
                    //             );
                    //             ui.label("Cutoff");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.sunspots_cutoff)
                    //                     .clamp_range(0.0f32..=1.0),
                    //             );
                    //             ui.label("Frequency");
                    //             ui.add(
                    //                 egui::DragValue::new(&mut star.sunspots_frequency).speed(0.001),
                    //             );
                    //             ui.label("Radius");
                    //         });
                    //     });

                    //     ui.horizontal(|ui| {
                    //         let mut color = egui::Color32::from_rgba_premultiplied(
                    //             (255.0 * star.color[0]) as u8,
                    //             (255.0 * star.color[1]) as u8,
                    //             (255.0 * star.color[2]) as u8,
                    //             (255.0 * star.color[3]) as u8,
                    //         );

                    //         ui.strong("Color");
                    //         egui::color_picker::color_picker_color32(
                    //             ui,
                    //             &mut color,
                    //             egui::color_picker::Alpha::Opaque,
                    //         );

                    //         star.color[0] = color[0] as f32 / 255 as f32;
                    //         star.color[1] = color[1] as f32 / 255 as f32;
                    //         star.color[2] = color[2] as f32 / 255 as f32;
                    //         star.color[3] = color[3] as f32 / 255 as f32;

                    //         let mut shifted_color = egui::Color32::from_rgba_premultiplied(
                    //             (255.0 * star.shift[0]) as u8,
                    //             (255.0 * star.shift[1]) as u8,
                    //             (255.0 * star.shift[2]) as u8,
                    //             (255.0 * star.shift[3]) as u8,
                    //         );

                    //         ui.strong("Shift");
                    //         egui::color_picker::color_picker_color32(
                    //             ui,
                    //             &mut shifted_color,
                    //             egui::color_picker::Alpha::Opaque,
                    //         );

                    //         star.shift[0] = shifted_color[0] as f32 / 255 as f32;
                    //         star.shift[1] = shifted_color[1] as f32 / 255 as f32;
                    //         star.shift[2] = shifted_color[2] as f32 / 255 as f32;
                    //         star.shift[3] = shifted_color[3] as f32 / 255 as f32;
                    //     });
                    // });
                });

                ui.add(&mut viewport);
            });

            // star_editor(&ctx, &mut universe.get_mut::<Star>(star).unwrap());

            // **********************
            // MAIN VIEWPORT ********
            // **********************

            universe_renderer.render(&universe, viewport.camera(), &renderer_settings);

            AppState::Run
        }
    });
}
