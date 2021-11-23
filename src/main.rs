pub mod app;

use log::info;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub use app::{App, Program};

fn main() {
    env_logger::init();
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Info)
    //     .init();

    info!("Initializing constellation engine...");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Constellation Engine")
        .build(&event_loop)
        .unwrap();

    let mut program = Program::new(window);

    event_loop.run(move |event, _loop, control_flow| {
        match event {
            Event::NewEvents(_) => program.on_new_events(),
            Event::WindowEvent { window_id, event } => program.on_window_event(window_id, &event),
            Event::DeviceEvent { device_id, event } => program.on_device_event(device_id, &event),
            Event::Suspended => program.on_suspend(),
            Event::Resumed => program.on_resume(),
            Event::MainEventsCleared => {
                if program.should_close() {
                    *control_flow = ControlFlow::Exit;
                }

                program.on_main_events_cleared();
            }
            Event::RedrawRequested(id) => program.on_redraw_event(id),
            Event::RedrawEventsCleared => program.on_redraw_events_cleared(),
            // other application-specific event handling
            _ => {
                // platform.handle_event(imgui.io_mut(), &window, &event); // step 3
                // other application-specific event handling
            }
        }
    })
}
