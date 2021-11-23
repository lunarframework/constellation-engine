pub mod canvas;
pub mod paint;
pub mod render;
pub mod ui_manager;

use canvas::Canvas;
use paint::{
    PaintData, PaintElement, PaintIdx, PaintVtx, Painter, Palette, PaletteDescriptor, PaletteId,
};
use render::Renderer;
use ui_manager::UiManager;

use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::window::{Window, WindowId};

use std::sync::Arc;

pub trait App {
    /// Create instance of App (running on the given window).
    fn new(window: Window) -> Self;

    /// Called before any events are processed for a given frame
    fn on_new_events(&mut self);

    // Main events

    fn on_window_event(&mut self, id: WindowId, event: &WindowEvent);

    fn on_device_event(&mut self, id: DeviceId, event: &DeviceEvent);

    fn on_suspend(&mut self);

    fn on_resume(&mut self);

    /// Called after all input events, but before any render events
    fn on_main_events_cleared(&mut self);

    // Redraw events

    /// Called after on_update if the window needs to be re-rendered.
    fn on_redraw_event(&mut self, id: WindowId);

    fn on_redraw_events_cleared(&mut self);

    fn should_close(&self) -> bool;
}

pub struct Program {
    // Window
    window: Arc<Window>,
    _renderer: Arc<Renderer>,

    // State tracking
    active: bool,
    should_close: bool,

    // Gui
    ui_manager: UiManager,

    // Rendering
    canvas: Canvas,
    painter: Painter,

    paint_data: PaintData,
}

impl App for Program {
    fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let renderer = Arc::new(Renderer::new());

        let canvas = Canvas::new(window.clone(), renderer.clone());
        let mut painter = Painter::new(renderer.clone());
        let mut ui_manager = UiManager::new(window.clone());
        ui_manager.reload_font_atlas(&mut painter);

        Self {
            window,
            _renderer: renderer,

            active: true,
            should_close: false,
            ui_manager,

            canvas,
            painter,

            paint_data: PaintData::new(),
        }
    }

    fn on_new_events(&mut self) {}

    fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        self.canvas.on_window_event(id, event);
        self.ui_manager.on_window_event(id, event);

        if self.window.id() == id {
            if let WindowEvent::CloseRequested = event {
                self.should_close = true;
            }
        }
    }

    fn on_device_event(&mut self, id: DeviceId, event: &DeviceEvent) {
        self.ui_manager.on_device_event(id, event);
    }

    fn on_suspend(&mut self) {
        self.active = false;
    }

    fn on_resume(&mut self) {
        self.active = true;
    }

    fn on_main_events_cleared(&mut self) {
        if self.active {
            self.ui_manager.frame(&mut self.paint_data, |ui| {
                // Build Ui here

                imgui::Window::new("Background Window")
                    .position([0.0, 0.0], imgui::Condition::Always)
                    .size(ui.io().display_size, imgui::Condition::Always)
                    .menu_bar(true)
                    .resizable(false)
                    .no_decoration()
                    .bring_to_front_on_focus(false)
                    .build(ui, || {
                        ui.menu_bar(|| {
                            ui.menu("File", || {});
                            ui.menu("Edit", || {});
                        });

                        ui.text("Welcome to Constellation Engine!");
                    });
            });

            self.window.request_redraw();
        }
    }

    fn on_redraw_event(&mut self, id: WindowId) {
        if self.canvas.id() == id {
            self.painter.paint(&self.canvas, &self.paint_data).unwrap();
        }
    }

    fn on_redraw_events_cleared(&mut self) {}

    fn should_close(&self) -> bool {
        self.should_close
    }
}
