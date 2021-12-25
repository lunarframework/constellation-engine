use egui::{CtxRef, Ui};

pub struct FileUi {
    enable_menu: bool,

    // New interface
    new: Option<NewSimulationUi>,
}

impl FileUi {
    pub fn new() -> Self {
        Self {
            enable_menu: false,

            new: None,
        }
    }

    pub fn append_menu(&mut self, ui: &mut Ui) {
        egui::menu::menu(ui, "File", |ui| {
            ui.group(|ui| {
                ui.set_enabled(self.enable_menu);
                if ui.button("New").clicked() {
                    self.new = Some(NewSimulationUi::new());
                }

                if ui.button("Open").clicked() {}
            });
        });

        self.enable_menu = self.new.is_none();
    }

    pub fn show_active_windows(&mut self, ctx: &CtxRef) {
        let mut opened = true;
        if let Some(ref mut new) = self.new {
            new.show(ctx, &mut opened);
        }

        if !(opened) {
            self.new = None;
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Dimenson {
    One,
    Two,
    Three,
}

#[derive(Debug, PartialEq, Eq)]
enum Method {
    FiniteDiff,
}

pub struct NewSimulationUi {
    name: String,
    dim: Dimenson,
    method: Method,
    rmax: f64,
    dr: f64,
}

impl NewSimulationUi {
    pub fn new() -> Self {
        Self {
            name: String::from("Untitled"),
            dim: Dimenson::One,
            method: Method::FiniteDiff,
            rmax: 1.0,
            dr: 0.5,
        }
    }

    pub fn show(&mut self, ctx: &CtxRef, opened: &mut bool) {
        egui::Window::new("New Simulation")
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
                            egui::DragValue::new(&mut self.rmax)
                                .speed(0.1)
                                .clamp_range(0.0..=10000.0),
                        );
                        ui.end_row();

                        ui.label("Delta R");
                        ui.add(
                            egui::DragValue::new(&mut self.dr)
                                .speed(0.1)
                                .clamp_range(0.0..=10000.0),
                        );
                        ui.end_row();
                    });
            });
    }
}
