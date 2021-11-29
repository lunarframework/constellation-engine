use super::State;

use egui::{
    plot::{Legend, Line, Plot, Value, Values},
    ComboBox, CtxRef, DragValue, SelectableLabel, Ui,
};

use ndarray::{s, Array, Array1, ArrayBase, ArrayView, ArrayView1, ArrayViewMut1, Zip};

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
    phi: Array1<f64>,
    psi: Array1<f64>,
    pi: Array1<f64>,

    // Ui state
    nr_plot: usize,
    dist: Dist,
}

impl InitialDataState {
    pub fn new(name: String, maxr: f64, nr: usize) -> Self {
        assert!(nr > 1);
        Self {
            name,
            maxr,
            nr,
            nr_plot: 100,
            dist: Dist::Flat { y: 0.0 },
            phi: Array1::zeros(nr),
            psi: Array1::zeros(nr),
            pi: Array1::zeros(nr),
        }
    }
}

impl State for InitialDataState {
    fn view(&mut self, ui: &mut Ui) {
        let dr = self.maxr / (self.nr - 1) as f64;
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
                    if ui.button("Apply").clicked() {
                        self.pi.fill(0.0);
                        match self.dist {
                            Dist::Flat { y } => {
                                self.phi.fill(y);
                                self.psi.fill(0.0);
                            }
                            Dist::Gaussian { amplitude, sigma } => {
                                for (i, (phi, psi)) in
                                    self.phi.iter_mut().zip(self.psi.iter_mut()).enumerate()
                                {
                                    let r = i as f64 * dr;

                                    *phi = amplitude * (-(r * r) / (sigma * sigma)).exp();
                                    *psi = -2.0 * r / (sigma * sigma) * *phi;
                                }
                            }
                        }
                    }
                });
                ui.group(|ui| {
                    ui.strong("View");
                    ui.separator();
                    // ui.add(DragValue::new(&mut self.nr_plot).clamp_range(2..=10000));
                })
            });
            ui.group(|ui| {
                let phi = Line::new(Values::from_values_iter(
                    self.phi
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| Value::new(i as f64 * dr, v)),
                ))
                .name("Scalar Field (Phi)");

                let psi = Line::new(Values::from_values_iter(
                    self.psi
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| Value::new(i as f64 * dr, v)),
                ))
                .name("Psi");

                ui.add(
                    Plot::new("Initial Data Plot")
                        .line(psi)
                        .line(phi)
                        .view_aspect(1.0)
                        .legend(Legend::default())
                        // .width(size.x)
                        // .height(size.y),
                );
            });
        });
    }
    fn show(&mut self, _ctx: &CtxRef) {}
    fn update(&mut self) {}
    fn title(&mut self) -> String {
        String::from("Initial Data Editor")
    }
}
