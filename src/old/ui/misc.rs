use imgui::{MenuItem, Ui, Window};

pub struct MiscUi {
    show_welcome: bool,
    show_metrics: bool,
    show_ui_demo: bool,
}

impl MiscUi {
    pub fn new() -> Self {
        Self {
            show_welcome: false,
            show_metrics: false,
            show_ui_demo: false,
        }
    }

    pub fn append_to_misc_menu(&mut self, ui: &Ui) {
        MenuItem::new("Metrics").build_with_ref(ui, &mut self.show_metrics);
        MenuItem::new("Welcome").build_with_ref(ui, &mut self.show_welcome);
        MenuItem::new("Ui Demo").build_with_ref(ui, &mut self.show_ui_demo);
    }

    pub fn show_active_windows(&mut self, ui: &Ui) {
        if self.show_metrics {
            Window::new("Metrics")
                .opened(&mut self.show_metrics)
                .build(ui, || {
                    ui.text(format!(
                        "Engine average {:.2} ms/frame ({:.2} FPS)",
                        1000.0 / ui.io().framerate,
                        ui.io().framerate
                    ));
                    ui.text(format!(
                        "{} vertices, {} indices ({} triangles)",
                        ui.io().metrics_render_vertices,
                        ui.io().metrics_render_indices,
                        ui.io().metrics_render_indices / 3
                    ));
                    ui.text(format!(
                        "{} windows ({} visible)",
                        ui.io().metrics_render_windows,
                        ui.io().metrics_active_windows
                    ));
                    ui.text(format!(
                        "{} active UI allocations",
                        ui.io().metrics_active_allocations
                    ));
                });
        }

        if self.show_welcome {
            Window::new("Welcome")
                .opened(&mut self.show_welcome)
                .build(ui, || {
                    ui.text("WELCOME TO CONSTELLATION ENGINE");
                    ui.separator();
                    ui.text_wrapped("   Constellation Engine is a tool for creating, editing, and visualizing physics simulations.
                                    It is developed by Lukas Mesicek, a physics undergrad at the University of Utah");
                });
        }

        if self.show_ui_demo {
            ui.show_demo_window(&mut self.show_ui_demo);
        }

        // Window::new("Welcome").
    }
}
