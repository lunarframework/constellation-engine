use crate::{InitialDataState, StateManager};
use egui::{Button, CtxRef, Ui};

pub struct FileMenu {
    // New interface
// new_dialog: Openable<NewDialog>,
// Simulate interface
// simulate_dialog: Option<SimulateDialog>,
}

impl FileMenu {
    pub fn new() -> Self {
        Self {
            // new_dialog: Openable::new(),
            // simulate_dialog: None,
        }
    }

    pub fn is_dialog_open(&self) -> bool {
        //self.new_dialog.is_open() ||
        // self.simulate_dialog.is_some()
        false
    }

    pub fn append(&mut self, ui: &mut Ui, manager: &mut StateManager) {
        egui::menu::menu(ui, "File", |ui| {
            if ui.button("New").clicked() {
                //self.new_dialog.open();
                manager.enqueue(InitialDataState::new());
            }

            ui.add_enabled(false, Button::new("Open"));

            ui.separator();

            // if ui.add_enabled(true, Button::new("Simulate")).clicked() {
            //     self.simulate_dialog = Some(SimulateDialog::new());
            // }

            ui.separator();
        });
    }

    pub fn show(&mut self, ctx: &CtxRef, manager: &mut StateManager) {
        // self.new_dialog.show(ctx, manager);
        // let mut opened = self.simulate_dialog.is_some();
        // if let Some(ref mut dialog) = self.simulate_dialog {
        //     Dialog::new("Simulate", &mut opened).show(ctx, |ui| dialog.ui(ui, manager));
        // }

        // if !opened {
        //     self.simulate_dialog.take();
        // }
    }
}

// #[derive(Debug, PartialEq, Eq)]
// enum Dimenson {
//     One,
//     Two,
//     Three,
// }

// #[derive(Debug, PartialEq, Eq)]
// enum Method {
//     FiniteDiff,
// }

pub struct NewDialog {
    name: String,
    // dim: Dimenson,
    // method: Method,
}

impl NewDialog {
    fn new() -> Self {
        Self {
            name: String::from("Untitled"),
            // dim: Dimenson::One,
            // method: Method::FiniteDiff
        }
    }

    fn name(&self) -> &str {
        "New Document"
    }

    fn ui(&mut self, ui: &mut Ui, manager: &mut StateManager) -> bool {
        ui.strong("Global Config").on_hover_ui(|ui| {
            ui.small("Global Settings of the simulation");
        });
        ui.separator();
        egui::Grid::new("Global Config Grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Simulation Name"));
                ui.end_row();

                // ui.label("Dimension");
                // egui::ComboBox::from_id_source("Dimension Combo Box")
                //     .selected_text(format!("{:?}", self.dim))
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(&mut self.dim, Dimenson::One, "One");
                //         ui.selectable_value(&mut self.dim, Dimenson::Two, "Two");
                //         ui.selectable_value(&mut self.dim, Dimenson::Three, "Three");
                //     });
                // ui.end_row();
            });

        ui.separator();

        // ui.strong("Finite Difference Config").on_hover_ui(|ui| {
        //     ui.small("How to store functions and integrate them forward in time");
        // });
        // ui.separator();
        // egui::Grid::new("Numerical Method Grid")
        //     .num_columns(2)
        //     .spacing([40.0, 4.0])
        //     .striped(true)
        //     .show(ui, |ui| {
        //         // egui::ComboBox::from_id_source("Method Combo Box")
        //         //     .selected_text(format!("{:?}", self.method))
        //         //     .show_ui(ui, |ui| {
        //         //         ui.selectable_value(
        //         //             &mut self.method,
        //         //             Method::FiniteDiff,
        //         //             "Finite Difference",
        //         //         );
        //         //     });
        //         // ui.end_row();
        //         // ui.label("Max Radius");
        //         // ui.add(
        //         //     egui::DragValue::new(&mut self.maxr)
        //         //         .speed(0.1)
        //         //         .clamp_range::<f64>(0.0..=10000.0),
        //         // );
        //         // ui.end_row();

        //         // ui.label("Num Grid Points");
        //         // ui.add(
        //         //     egui::DragValue::new(&mut self.nr)
        //         //         .speed(0.1)
        //         //         .clamp_range::<usize>(2..=10000),
        //         // );
        //         // ui.end_row();
        //     });

        // ui.separator();
        if ui.button("Create").clicked() {
            manager.enqueue(InitialDataState::new());
            return false;
        };

        true
    }
}

pub struct SimulateDialog {
    rmax: f64,
    tend: f64,
    sampled_points: usize,
    iterations: usize,
    compression: usize,
}

impl SimulateDialog {
    fn new() -> Self {
        Self {
            rmax: 1.0,
            tend: 1.0,
            sampled_points: 2,
            iterations: 2,
            compression: 1,
        }
    }

    fn ui(&mut self, ui: &mut Ui, _manager: &mut StateManager) -> bool {
        ui.group(|ui| {
            ui.strong("Domain").on_hover_ui(|ui| {
                ui.small("Settings for the coordinate domain");
            });

            ui.separator();

            egui::Grid::new("Domain Grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("RMin");
                    ui.label("0.0 m");
                    ui.end_row();

                    ui.label("RMax");
                    ui.add(egui::DragValue::new(&mut self.rmax).speed(1.0).suffix("m"));
                    ui.end_row();

                    ui.label("TBegin");
                    ui.label("0.0 s");
                    ui.end_row();

                    ui.label("TEnd");
                    ui.add(egui::DragValue::new(&mut self.tend).speed(1.0).suffix("s"));
                    ui.end_row();
                });
        });

        ui.group(|ui| {
            ui.strong("Accuracy").on_hover_ui(|ui| {
                ui.small("Settings for the percision of the simulation");
            });

            ui.separator();

            egui::Grid::new("Accuracy Grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Sampled Points");
                    ui.add(egui::DragValue::new(&mut self.sampled_points));
                    ui.end_row();

                    ui.label("Iterations");
                    ui.add(egui::DragValue::new(&mut self.iterations));
                    ui.end_row();
                });

            ui.separator();

            let delta_r = self.rmax / self.sampled_points as f64;
            let delta_t = self.tend / self.iterations as f64;

            ui.label(format!("Delta R: {} m", delta_r));
            ui.label(format!("Delta T: {} s", delta_t));
            ui.label(format!("Courant Factor: {}", delta_t / delta_r));
        });

        ui.group(|ui| {
            ui.strong("Storage").on_hover_ui(|ui| {
                ui.small("Settings for how the simulation is stored");
            });

            ui.separator();

            egui::Grid::new("Storage Grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Compression");
                    ui.add(egui::DragValue::new(&mut self.compression));
                    ui.end_row();
                });
        });

        // ui.separator();
        if ui.button("Simulate").clicked() {
            // manager.enqueue(InitialDataState::new(self.name.clone()));
            return false;
        };

        true
    }
}
