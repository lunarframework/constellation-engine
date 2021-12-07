use super::State;
use egui::{
    plot::{Legend, Line, Plot, Value, Values},
    ComboBox, CtxRef, DragValue, SelectableLabel, Ui,
};
use ndarray::{s, Array, Array1, ArrayBase, ArrayView, ArrayView1, ArrayViewMut1, Zip};
use std::ops::RangeInclusive;

enum Dist {
    Flat { y: f64 },
    Gaussian { amplitude: f64, sigma: f64 },
}

impl Dist {
    fn to_string(&self) -> &'static str {
        match self {
            Dist::Flat { .. } => "Flat",
            Dist::Gaussian { .. } => "Gaussian",
        }
    }

    fn is_flat(&self) -> bool {
        if let Dist::Flat { .. } = self {
            true
        } else {
            false
        }
    }

    fn is_gaussian(&self) -> bool {
        if let Dist::Gaussian { .. } = self {
            true
        } else {
            false
        }
    }
}

pub struct InitialDataState {
    name: String,
    maxr: f64,
    nr: usize,
    dist: Dist,
    show_psi: bool,
    show_pi: bool,
}

impl InitialDataState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            maxr: 1.0,
            nr: 100,
            dist: Dist::Flat { y: 0.0 },
            show_psi: false,
            show_pi: false,
        }
    }
}

impl State for InitialDataState {
    fn view(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_max_width(available_size.x * 1.0 / 5.0);
                ui.group(|ui| {
                    ui.strong("Data");
                    ui.separator();
                    ui.label("Distribution");
                    ComboBox::from_id_source("Distribution Combo Box")
                        .selected_text(self.dist.to_string())
                        .show_ui(ui, |ui| {
                            if ui
                                .add(SelectableLabel::new(self.dist.is_flat(), "Flat"))
                                .clicked()
                            {
                                self.dist = Dist::Flat { y: 0.0 };
                            }

                            if ui
                                .add(SelectableLabel::new(self.dist.is_gaussian(), "Gaussian"))
                                .clicked()
                            {
                                self.dist = Dist::Gaussian {
                                    amplitude: 1.0,
                                    sigma: 1.0,
                                };
                            }
                        });

                    ui.separator();

                    match self.dist {
                        Dist::Flat { ref mut y } => {
                            ui.label("Y-value");
                            ui.add(DragValue::new(y).speed(0.1));
                        }
                        Dist::Gaussian {
                            ref mut amplitude,
                            ref mut sigma,
                        } => {
                            ui.label("Amplitude");
                            ui.add(DragValue::new(amplitude).speed(0.1));
                            ui.label("Sigma");
                            ui.add(DragValue::new(sigma).speed(0.1));
                        }
                    }

                    ui.separator();
                });
                ui.group(|ui| {
                    ui.strong("View");
                    ui.separator();
                    ui.label("Maximum Radius");
                    ui.add(
                        DragValue::new(&mut self.maxr)
                            .speed(0.1)
                            .clamp_range::<f64>(0.0..=100000.0),
                    );
                    ui.label("Num Points");
                    ui.add(DragValue::new(&mut self.nr).clamp_range(2..=100000));

                    ui.separator();

                    ui.checkbox(&mut self.show_psi, "Graph Psi");
                    ui.checkbox(&mut self.show_pi, "Graph Pi");
                    // ui.add(DragValue::new(&mut self.nr_plot).clamp_range(2..=10000));
                });
                ui.group(|ui| if ui.button("Simulate").clicked() {})
            });
            ui.group(|ui| {
                let pi = Line::new(Values::from_explicit_callback(|_r| 0.0, 0.0..self.maxr, 2));
                let phi: Line;
                let psi: Line;

                match self.dist {
                    Dist::Flat { y } => {
                        phi = Line::new(Values::from_explicit_callback(
                            move |_r| y,
                            0.0..self.maxr,
                            2,
                        ));

                        psi =
                            Line::new(Values::from_explicit_callback(|_r| 0.0, 0.0..self.maxr, 2));
                    }
                    Dist::Gaussian { amplitude, sigma } => {
                        phi = Line::new(Values::from_explicit_callback(
                            move |r| amplitude * (-(r * r) / (sigma * sigma)).exp(),
                            0.0..self.maxr,
                            self.nr,
                        ));

                        psi = Line::new(Values::from_explicit_callback(
                            move |r| {
                                -2.0 * r / (sigma * sigma)
                                    * amplitude
                                    * (-(r * r) / (sigma * sigma)).exp()
                            },
                            0.0..self.maxr,
                            self.nr,
                        ));
                    }
                }

                let mut plot = Plot::new("Initial Data Plot")
                    .line(phi.name("Phi (Scalar Field)"))
                    .data_aspect(1.0)
                    .view_aspect(1.0)
                    .legend(Legend::default());

                if self.show_psi {
                    plot = plot.line(psi.name("Psi"));
                }

                if self.show_pi {
                    plot = plot.line(pi.name("Pi"));
                }

                ui.add(plot);
            });
        });
    }
    fn show(&mut self, _ctx: &CtxRef) {}
    fn update(&mut self) {}
    fn title(&mut self) -> &str {
        "Initial Data Editor"
    }
}
