use crate::{GuiHandler, HiDpiMode, Renderer};

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

    // State tracking
    active: bool,
    should_close: bool,

    renderer: Renderer,
    gui_handler: GuiHandler,
}

impl App for Program {
    fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let gui_handler = GuiHandler::new(window.clone(), HiDpiMode::Rounded);
        let renderer = Renderer::new(window.clone());

        Self {
            window,
            active: true,
            should_close: false,
            gui_handler,
            renderer,
        }
    }

    fn on_new_events(&mut self) {}

    fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        self.gui_handler.on_window_event(id, event);
        self.renderer.on_window_event(id, event);

        if self.window.id() == id {
            if let WindowEvent::CloseRequested = event {
                self.should_close = true;
            }
        }
    }

    fn on_device_event(&mut self, id: DeviceId, event: &DeviceEvent) {
        self.gui_handler.on_device_event(id, event);
    }

    fn on_suspend(&mut self) {
        self.active = false;
    }

    fn on_resume(&mut self) {
        self.active = true;
    }

    fn on_main_events_cleared(&mut self) {
        if self.active {
            // let ui = self.gui_handler.frame();

            //  Build Ui here

            // let _draw_data = ui.render();

            self.window.request_redraw();
        }
    }

    fn on_redraw_event(&mut self, id: WindowId) {
        self.renderer.on_redraw_event(id);
    }

    fn on_redraw_events_cleared(&mut self) {}

    fn should_close(&self) -> bool {
        self.should_close
    }
}
