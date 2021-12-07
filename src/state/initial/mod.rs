use super::State;

use egui::{
    plot::{Legend, Line, Plot, Values},
    ComboBox, CtxRef, DragValue, SelectableLabel, Ui,
};

#[derive(Clone)]
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

use crate::physics::scalar;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread::JoinHandle;

struct Simulator {
    task: Option<JoinHandle<()>>,
    receiver: Receiver<f64>,
    progress: f64,
}

impl Simulator {
    fn new(
        rmax: f64,
        points: usize,
        tend: f64,
        iterations: usize,
        mass: f64,
        dist: Dist,
    ) -> Simulator {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut wave = scalar::Function1::zeros(points);

        match dist {
            Dist::Flat { y } => {
                wave.fill(y);
            }
            Dist::Gaussian { amplitude, sigma } => {
                for i in 0..points {
                    let r = i as f64 / (points - 1) as f64 * rmax;
                    wave[i] = amplitude * (-(r * r) / (sigma * sigma)).exp();
                }
            }
        };

        let data = scalar::InitialData {
            rmax,
            wave,
            tend,
            iterations,
            mass,
        };
        let task = std::thread::spawn(move || {
            scalar::simulate(data, |t| {
                sender.send(t / tend).unwrap();
            });
        });

        Simulator {
            task: Some(task),
            receiver,
            progress: 0.0,
        }
    }

    fn update(&mut self) -> bool {
        loop {
            match self.receiver.try_recv() {
                Ok(p) => {
                    self.progress = p;
                }
                Err(err) => match err {
                    TryRecvError::Empty => {
                        return false;
                    }
                    TryRecvError::Disconnected => {
                        break;
                    }
                },
            }
        }

        self.task.take().unwrap().join().unwrap();

        true
    }
}

pub struct InitialDataState {
    // Data
    rmax: f64,
    points: usize,
    dist: Dist,
    mass: f64,
    // UI
    show_psi: bool,
    show_pi: bool,

    // Simulation settings (Domain)
    tend: f64,
    iterations: usize,
    // Simulation settings (strorage)
    _sim_compression: usize,

    simulator: Option<Simulator>,
}

impl InitialDataState {
    pub fn new() -> Self {
        Self {
            points: 100,
            rmax: 1.0,
            dist: Dist::Flat { y: 0.0 },
            mass: 1.0,

            show_psi: false,
            show_pi: false,

            tend: 1.0,
            iterations: 100,
            _sim_compression: 1,

            simulator: None,
        }
    }
}

impl State for InitialDataState {
    fn view(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();

        ui.add_enabled_ui(self.simulator.is_none(), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_max_width(available_size.x * 1.0 / 5.0);
                    ui.group(|ui| {
                        ui.strong("Data");
                        ui.separator();
                        egui::Grid::new("Data Grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
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
                                            .add(SelectableLabel::new(
                                                self.dist.is_gaussian(),
                                                "Gaussian",
                                            ))
                                            .clicked()
                                        {
                                            self.dist = Dist::Gaussian {
                                                amplitude: 1.0,
                                                sigma: 1.0,
                                            };
                                        }
                                    });
                                ui.end_row();
                                match self.dist {
                                    Dist::Flat { ref mut y } => {
                                        ui.label("Y-value");
                                        ui.add(DragValue::new(y).speed(0.1));
                                        ui.end_row();
                                    }
                                    Dist::Gaussian {
                                        ref mut amplitude,
                                        ref mut sigma,
                                    } => {
                                        ui.label("Amplitude");
                                        ui.add(DragValue::new(amplitude).speed(0.1));
                                        ui.end_row();

                                        ui.label("Sigma");
                                        ui.add(DragValue::new(sigma).speed(0.1));
                                        ui.end_row();
                                    }
                                }

                                ui.label("Max Radius");
                                ui.add(
                                    DragValue::new(&mut self.rmax)
                                        .speed(0.1)
                                        .clamp_range::<f64>(0.0..=100000.0),
                                );
                                ui.end_row();

                                ui.label("Points");
                                ui.add(
                                    DragValue::new(&mut self.points)
                                        .speed(1)
                                        .clamp_range::<usize>(4..=100000),
                                );
                                ui.end_row();

                                ui.label("Mass");
                                ui.add(
                                    DragValue::new(&mut self.mass)
                                        .speed(0.1)
                                        .clamp_range::<f64>(0.0..=100000.0),
                                );
                                ui.end_row();
                            });
                        ui.separator();
                    });
                    // ui.group(|ui| {
                    //     ui.strong("View").on_hover_ui(|ui| {
                    //         ui.small("Settings for the graph viewport");
                    //     });
                    //     ui.separator();

                    //     egui::Grid::new("View Grid")
                    //         .num_columns(2)
                    //         .spacing([40.0, 4.0])
                    //         .striped(true)
                    //         .show(ui, |ui| {
                    //             ui.label("View Points");
                    //             ui.add(DragValue::new(&mut self.nr).clamp_range(2..=100000));
                    //             ui.end_row();
                    //         });

                    //     ui.separator();

                    //     ui.checkbox(&mut self.show_psi, "Graph Psi");
                    //     ui.checkbox(&mut self.show_pi, "Graph Pi");
                    // });
                    ui.group(|ui| {
                        ui.strong("Simulate").on_hover_ui(|ui| {
                            ui.small("Settings for simulation");
                        });

                        ui.separator();

                        egui::Grid::new("Simulate Grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Time End");
                                ui.add(egui::DragValue::new(&mut self.tend).speed(1.0).suffix("s"));
                                ui.end_row();

                                ui.label("Iterations");
                                ui.add(
                                    egui::DragValue::new(&mut self.iterations)
                                        .speed(1)
                                        .clamp_range::<usize>(2..=10000000),
                                );
                                ui.end_row();

                                // ui.label("Compression");
                                // ui.add(
                                //     egui::DragValue::new(&mut self.sim_compression)
                                //         .speed(1)
                                //         .clamp_range(1..=10000),
                                // );
                                // ui.end_row();
                            });
                        ui.separator();

                        if ui.button("Simulate").clicked() {
                            self.simulator = Some(Simulator::new(
                                self.rmax,
                                self.points,
                                self.tend,
                                self.iterations,
                                self.mass,
                                self.dist.clone(),
                            ));
                        }
                    });
                });
                ui.group(|ui| {
                    let pi = Line::new(Values::from_explicit_callback(|_r| 0.0, 0.0..self.rmax, 2));
                    let phi: Line;
                    let psi: Line;

                    match self.dist {
                        Dist::Flat { y } => {
                            phi = Line::new(Values::from_explicit_callback(
                                move |_r| y,
                                0.0..self.rmax,
                                2,
                            ));

                            psi = Line::new(Values::from_explicit_callback(
                                |_r| 0.0,
                                0.0..self.rmax,
                                2,
                            ));
                        }
                        Dist::Gaussian { amplitude, sigma } => {
                            phi = Line::new(Values::from_explicit_callback(
                                move |r| amplitude * (-(r * r) / (sigma * sigma)).exp(),
                                0.0..=self.rmax,
                                self.points,
                            ));

                            psi = Line::new(Values::from_explicit_callback(
                                move |r| {
                                    -2.0 * r / (sigma * sigma)
                                        * amplitude
                                        * (-(r * r) / (sigma * sigma)).exp()
                                },
                                0.0..=self.rmax,
                                self.points,
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
        });
    }
    fn show(&mut self, ctx: &CtxRef) {
        if let Some(ref simulator) = self.simulator {
            egui::Window::new("Simulating...")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.add(egui::ProgressBar::new(simulator.progress as f32).show_percentage());
                });
        }
    }
    fn update(&mut self) {
        let mut finish_simulation = false;
        if let Some(ref mut simulator) = self.simulator {
            finish_simulation = simulator.update();
        }

        if finish_simulation {
            self.simulator.take();
            println!("Finished simulation!");
        }
    }
    fn title(&mut self) -> &str {
        "Initial Data Editor"
    }
}
