pub mod frame;
pub mod render;

use frame::Framework;
use render::Renderer;

use winit::event::Event;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::window::WindowId;

use egui::CtxRef;

use std::sync::Arc;

pub trait ConsoleApp: 'static {
    /// Called before any events are processed for a given frame
    fn on_new_events(&mut self) {}

    // Callbacks

    fn on_window_event(&mut self, _id: WindowId, _event: &WindowEvent) {}

    fn on_device_event(&mut self, _id: DeviceId, _event: &DeviceEvent) {}

    fn on_suspend(&mut self) {}

    fn on_resume(&mut self) {}

    fn on_main_events_cleared(&mut self) {}

    fn on_redraw_event(&mut self, _id: WindowId) {}

    fn on_redraw_events_cleared(&mut self) {}

    // Additional options

    fn should_close(&mut self) -> bool {
        false
    }

    fn title(&mut self) -> String {
        String::from("Constellation Engine")
    }

    // Main loop

    fn update(&mut self, ctx: &CtxRef);
}

pub fn launch(mut app: impl ConsoleApp) {
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Constellation Engine")
            .build(&event_loop)
            .unwrap(),
    );

    let renderer = Arc::new(Renderer::new());

    let mut frame = Framework::new(
        window.clone(),
        renderer.clone(),
        wgpu::TextureFormat::Bgra8Unorm,
    );

    let mut active = true;
    let mut should_close = false;

    event_loop.run(move |event, _loop, control_flow| {
        match event {
            Event::NewEvents(_) => app.on_new_events(),
            Event::WindowEvent { window_id, event } => {
                frame.on_window_event(window_id, &event);

                if window.id() == window_id {
                    if let WindowEvent::CloseRequested = event {
                        should_close = true;
                    }
                }

                app.on_window_event(window_id, &event);
            }
            Event::DeviceEvent { device_id, event } => {
                frame.on_device_event(device_id, &event);
                app.on_device_event(device_id, &event);
            }
            Event::Suspended => {
                active = false;
                app.on_suspend();
            }
            Event::Resumed => {
                active = true;
                app.on_resume();
            }
            Event::MainEventsCleared => {
                if should_close || app.should_close() {
                    *control_flow = ControlFlow::Exit;
                }

                if active {
                    window.request_redraw();
                }

                app.on_main_events_cleared();
            }
            Event::RedrawRequested(id) => {
                if window.id() == id && active {
                    frame.begin_frame();
                    app.update(&frame.context());
                    frame.end_frame().unwrap();

                    window.set_title(app.title().as_ref());
                }

                app.on_redraw_event(id);
            }
            Event::RedrawEventsCleared => app.on_redraw_events_cleared(),
            // other application-specific event handling
            _ => {
                // platform.handle_event(imgui.io_mut(), &window, &event); // step 3
                // other application-specific event handling
            }
        }
    })
}
