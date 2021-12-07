pub struct SimulateDialog {
    rmax: f64,
    tend: f64,
    sampled_points: usize,
    iterations: usize,
    compression: usize,
}

impl Dialog for SimulateDialog {
    fn new() -> Self {
        Self {
            rmax: 1.0,
            tend: 1.0,
            sampled_points: 2,
            iterations: 2,
            compression: 1,
        }
    }

    fn name(&self) -> &str {
        "Simulate"
    }

    fn show(&mut self, ui: &mut Ui, _manager: &mut StateManager) -> bool {
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
            return true;
        };

        false
    }
}
