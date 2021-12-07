use crate::physics::scalar;
use egui::{CtxRef, Ui};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread::JoinHandle;

pub struct State {
    page: Page,
}

impl State {
    pub fn new() -> Self {
        Self { page: Page::Home }
    }

    pub fn show(&mut self, ctx: &CtxRef) {
        egui::TopBottomPanel::top("Menu Bar Panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("New").clicked() {
                        self.page = Page::InitialData {
                            rmax: 1.0,
                            points: 100,
                            dist: Distribution::Flat { y: 0.0 },
                            mass: 1.0,
                            show_psi: false,
                            show_pi: false,
                            tend: 1.0,
                            iterations: 100,
                            simulator: None,
                        }
                    }

                    // ui.add_enabled(false, egui::Button::new("Open"));

                    // ui.separator();

                    // if ui.add_enabled(true, Button::new("Simulate")).clicked() {
                    //     self.simulate_dialog = Some(SimulateDialog::new());
                    // }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Home => {
                ui.strong("WELCOME TO CONSTELLATION ENGINE");
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations. It is developed by Lukas Mesicek, a physics undergrad at the University of Utah."); 
                });
            },
            Page::InitialData { ref mut dist, ref mut rmax, ref mut points, ref mut mass, ref mut tend, ref mut iterations, ref mut show_psi, ref mut show_pi, ref mut simulator,.. } => {
                let available_size = ui.available_size();

                // Begin Enabled Ui
                ui.add_enabled_ui(simulator.is_none(), |ui| {
                    // Begin Horizontal
                    ui.horizontal(|ui| {
                        // Begin Vertical
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
                                            egui::ComboBox::from_id_source("Distribution Combo Box")
                                                .selected_text(dist.to_string())
                                                .show_ui(ui, |ui| {
                                                    if ui
                                                        .add(egui::SelectableLabel::new(dist.is_flat(), "Flat"))
                                                        .clicked()
                                                    {
                                                        *dist = Distribution::Flat { y: 0.0 };
                                                    }

                                                    if ui
                                                        .add(egui::SelectableLabel::new(
                                                            dist.is_gaussian(),
                                                            "Gaussian",
                                                        ))
                                                        .clicked()
                                                    {
                                                        *dist = Distribution::Gaussian {
                                                            amplitude: 1.0,
                                                            sigma: 1.0,
                                                        };
                                                    }
                                                });    
                                            ui.end_row();
                                            match dist {
                                                Distribution::Flat { y } => {
                                                    ui.label("Y-value");
                                                    ui.add(egui::DragValue::new(y).speed(0.1));
                                                    ui.end_row();
                                                }
                                                Distribution::Gaussian {
                                                    amplitude,
                                                    sigma,
                                                } => {
                                                    ui.label("Amplitude");
                                                    ui.add(egui::DragValue::new(amplitude).speed(0.1));
                                                    ui.end_row();

                                                    ui.label("Sigma");
                                                    ui.add(egui::DragValue::new(sigma).speed(0.1));
                                                    ui.end_row();
                                                }
                                            }

                                            ui.label("Max Radius");
                                            ui.add(
                                                egui::DragValue::new(rmax)
                                                    .speed(0.1)
                                                    .clamp_range::<f64>(0.0..=100000.0),
                                            );
                                            ui.end_row();

                                            ui.label("Points");
                                            ui.add(
                                                egui::DragValue::new(points)
                                                    .speed(1)
                                                    .clamp_range::<usize>(4..=100000),
                                            );
                                            ui.end_row();

                                            ui.label("Mass");
                                            ui.add(
                                                egui::DragValue::new(mass)
                                                    .speed(0.1)
                                                    .clamp_range::<f64>(0.0..=100000.0),
                                            );
                                            ui.end_row();
                                        });
                                    ui.separator();
                                });
                            // Begin Simulate Group    
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
                                        ui.add(egui::DragValue::new(tend).speed(1.0).suffix("s"));
                                        ui.end_row();

                                        ui.label("Iterations");
                                        ui.add(
                                            egui::DragValue::new(iterations)
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
                                    *simulator = Some(SimulatorHandle::new(
                                        *rmax,
                                        *points,
                                        *tend,
                                        *iterations,
                                        *mass,
                                        dist.clone(),
                                    ));
                                }
                            }); // End simulate group
                        }); // End vertical

                        // Begin Plot group
                        ui.group(|ui| {

                            use egui::plot::{Line, Values, Legend, Plot};

                            let pi = Line::new(Values::from_explicit_callback(|_r| 0.0, 0.0..*rmax, 2));
                            let phi: Line;
                            let psi: Line;

                            match *dist {
                                Distribution::Flat { y } => {
                                    phi = Line::new(Values::from_explicit_callback(
                                        move |_r| y,
                                        0.0..*rmax,
                                        2,
                                    ));

                                    psi = Line::new(Values::from_explicit_callback(
                                        |_r| 0.0,
                                        0.0..*rmax,
                                        2,
                                    ));
                                }
                                Distribution::Gaussian { amplitude, sigma } => {
                                    phi = Line::new(Values::from_explicit_callback(
                                        move |r| amplitude * (-(r * r) / (sigma * sigma)).exp(),
                                        0.0..=*rmax,
                                        *points,
                                    ));

                                    psi = Line::new(Values::from_explicit_callback(
                                        move |r| {
                                            -2.0 * r / (sigma * sigma)
                                                * amplitude
                                                * (-(r * r) / (sigma * sigma)).exp()
                                        },
                                        0.0..=*rmax,
                                        *points,
                                    ));
                                }
                            }

                            let mut plot = Plot::new("Initial Data Plot")
                                .line(phi.name("Phi (Scalar Field)"))
                                .data_aspect(1.0)
                                .view_aspect(1.0)
                                .legend(Legend::default());

                            if *show_psi {
                                plot = plot.line(psi.name("Psi"));
                            }

                            if *show_pi {
                                plot = plot.line(pi.name("Pi"));
                            }

                            ui.add(plot);
                        }); // End plot group
                    }); // End horizontal
                }); // End enabled Ui
            },
            Page::Simulation { ref mut sim, ref mut time } => {
                // Begin Plot group
                ui.group(|ui| {

                    use egui::plot::{Line, Values, Legend, Plot};

                    let phi = &sim.phi;

                    let time_len = phi.dim().0;
                    let spatial_len = phi.dim().1;
                    let fraction = *time / sim.tend;
                    let i = fraction * (time_len - 1) as f64;

                    let (ilower, iupper) = (i.floor() as usize, i.ceil() as usize);

                    let tinterp = i - i.floor();

                    let lower = phi.index_axis(ndarray::Axis(0), ilower);
                    let upper = phi.index_axis(ndarray::Axis(0), iupper);

                    // println!("tinterp {}, lower len {}, upper len {}, spatial len {}", tinterp, lower.len(), upper.len(), spatial_len);

                    let phi = Line::new(Values::from_explicit_callback(|_r| 0.0, 0.0..sim.rmax, 2));

                    let plot = Plot::new("Simulation Plot")
                        .line(phi.name("Phi (Scalar Field)"))
                        .data_aspect(1.0)
                        .view_aspect(1.0)
                        .legend(Legend::default());

                    ui.add(plot);

                    ui.add(egui::Slider::new(time, 0.0..=sim.tend).text("Time (s)"));
                    
                }); // End plot group
            }
        });

        match self.page {
            Page::InitialData { ref mut simulator, .. } => {
                if let Some(ref simulator) = simulator {
                    egui::Window::new("Simulating...")
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.add(egui::ProgressBar::new(simulator.progress as f32).show_percentage());
                        });
                }
            },
            _ => {}
        }
    }

    pub fn update(&mut self) {
        match self.page {
            Page::InitialData {
                ref mut simulator, ..
            } => {
                if let Some(ref mut sim) = simulator {
                    if let Some(res) = sim.poll() {
                        simulator.take();
                        self.page = Page::Simulation {
                            sim: res,
                            time: 0.0
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn title(&mut self) -> &str {
        match self.page {
            Page::Home => "Welcome",
            Page::InitialData { .. } => "Initial Data Editor",
            Page::Simulation { .. } => "Simulation Viewer",
        }
    }
}

enum Page {
    Home,
    InitialData {
        // Data
        rmax: f64,
        points: usize,
        dist: Distribution,
        mass: f64,
        // View
        show_psi: bool,
        show_pi: bool,

        // Simulation
        tend: f64,
        iterations: usize,

        simulator: Option<SimulatorHandle>,
    },
    Simulation {
        sim: scalar::Simulation,
        time: f64,
    },
}

#[derive(Clone)]
enum Distribution {
    Flat { y: f64 },
    Gaussian { amplitude: f64, sigma: f64 },
}

impl Distribution {
    fn to_string(&self) -> &'static str {
        match self {
            Distribution::Flat { .. } => "Flat",
            Distribution::Gaussian { .. } => "Gaussian",
        }
    }

    fn is_flat(&self) -> bool {
        if let Distribution::Flat { .. } = self {
            true
        } else {
            false
        }
    }

    fn is_gaussian(&self) -> bool {
        if let Distribution::Gaussian { .. } = self {
            true
        } else {
            false
        }
    }
}

struct SimulatorHandle {
    task: Option<JoinHandle<scalar::Simulation>>,
    receiver: Receiver<f64>,
    progress: f64,
}

impl SimulatorHandle {
    fn new(
        rmax: f64,
        points: usize,
        tend: f64,
        iterations: usize,
        mass: f64,
        dist: Distribution,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut wave = scalar::Function1::zeros(points);

        match dist {
            Distribution::Flat { y } => {
                wave.fill(y);
            }
            Distribution::Gaussian { amplitude, sigma } => {
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
            })
        });

        Self {
            task: Some(task),
            receiver,
            progress: 0.0,
        }
    }

    fn poll(&mut self) -> Option<scalar::Simulation> {
        loop {
            match self.receiver.try_recv() {
                Ok(p) => {
                    self.progress = p;
                }
                Err(err) => match err {
                    TryRecvError::Empty => {
                        return None;
                    }
                    TryRecvError::Disconnected => {
                        break;
                    }
                },
            }
        }

        Some(self.task.take().unwrap().join().unwrap())
    }
}
