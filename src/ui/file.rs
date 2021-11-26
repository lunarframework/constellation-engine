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

pub struct NewSimulationUi {
    name: String,
    dim: Dimenson,
}

impl NewSimulationUi {
    pub fn new() -> Self {
        Self {
            name: String::from("Untitled"),
            dim: Dimenson::One,
        }
    }

    pub fn show(&mut self, ctx: &CtxRef, opened: &mut bool) {
        egui::Window::new("New Simulation")
            .open(opened)
            .resizable(true)
            .default_width(300.0)
            .auto_sized()
            .show(ctx, |ui| {
                ui.label("Global Config");
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

                        ui.label("Dimension");
                        egui::ComboBox::from_id_source("Dimension Combo Box")
                            .selected_text(format!("{:?}", self.dim))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.dim, Dimenson::One, "One");
                                ui.selectable_value(&mut self.dim, Dimenson::Two, "Two");
                                ui.selectable_value(&mut self.dim, Dimenson::Three, "Three");
                            });
                        ui.end_row();
                    });
            });
    }
}
