use egui::{CtxRef, Ui};

use crate::InitialDataState;
use crate::StateManager;

pub struct FileMenu {
    enable: bool,

    // New interface
    new_dialog: Option<NewDialog>,
}

impl FileMenu {
    pub fn new() -> Self {
        Self {
            enable: false,

            new_dialog: None,
        }
    }

    pub fn append(&mut self, ui: &mut Ui, _manager: &mut StateManager) {
        egui::menu::menu(ui, "File", |ui| {
            ui.group(|ui| {
                ui.set_enabled(self.enable);
                if ui.button("New").clicked() {
                    self.new_dialog = Some(NewDialog::new());
                }

                if ui.button("Open").clicked() {}
            });
        });

        self.enable = self.new_dialog.is_none();
    }

    pub fn show(&mut self, ctx: &CtxRef, manager: &mut StateManager) {
        let mut opened = true;
        if let Some(ref mut dialog) = self.new_dialog {
            dialog.show(ctx, manager, &mut opened);
        }

        if !(opened) {
            self.new_dialog = None;
        }
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
    maxr: f64,
    nr: usize,
}

impl NewDialog {
    pub fn new() -> Self {
        Self {
            name: String::from("Untitled"),
            // dim: Dimenson::One,
            // method: Method::FiniteDiff,
            maxr: 1.0,
            nr: 2,
        }
    }

    pub fn show(&mut self, ctx: &CtxRef, manager: &mut StateManager, opened: &mut bool) {
        let mut should_close = false;
        egui::Window::new("New Document")
            .open(opened)
            .resizable(true)
            .default_width(300.0)
            .auto_sized()
            .show(ctx, |ui| {
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
                        ui.add(
                            egui::TextEdit::singleline(&mut self.name).hint_text("Simulation Name"),
                        );
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

                ui.strong("Finite Difference Config").on_hover_ui(|ui| {
                    ui.small("How to store functions and integrate them forward in time");
                });
                ui.separator();
                egui::Grid::new("Numerical Method Grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // egui::ComboBox::from_id_source("Method Combo Box")
                        //     .selected_text(format!("{:?}", self.method))
                        //     .show_ui(ui, |ui| {
                        //         ui.selectable_value(
                        //             &mut self.method,
                        //             Method::FiniteDiff,
                        //             "Finite Difference",
                        //         );
                        //     });
                        // ui.end_row();
                        ui.label("Max Radius");
                        ui.add(
                            egui::DragValue::new(&mut self.maxr)
                                .speed(0.1)
                                .clamp_range::<f64>(0.0..=10000.0),
                        );
                        ui.end_row();

                        ui.label("Num Grid Points");
                        ui.add(
                            egui::DragValue::new(&mut self.nr)
                                .speed(0.1)
                                .clamp_range::<usize>(2..=10000),
                        );
                        ui.end_row();
                    });

                ui.separator();
                if ui.button("Create").clicked() {
                    manager.enqueue(InitialDataState::new(self.name.clone(), self.maxr, self.nr));
                    should_close = true;
                }
            });

        *opened &= !should_close;
    }
}
