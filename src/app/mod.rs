pub mod frame;
pub mod render;

pub use frame::Framework;
pub use render::Renderer;

use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;

use std::borrow::Borrow;

use egui::CtxRef;

use std::sync::Arc;

pub enum AppEvent<'a> {
    CloseRequested,

    // TODO this is temporary. Eventually Framework (or at least rendering related functions) will be moved into AppContext
    Frame {
        ctx: CtxRef,
        frame: &'a mut Framework,
    },
}

pub enum AppState {
    Run,
    Exit,
}

#[derive(Clone)]
pub struct AppContex {
    window: Arc<Window>,
    renderer: Arc<Renderer>,
}

impl AppContex {
    pub fn window(&self) -> &Window {
        self.window.borrow()
    }

    pub fn renderer(&self) -> &Renderer {
        self.renderer.borrow()
    }
}

pub struct App {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    renderer: Arc<Renderer>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Constellation Engine")
                .build(&event_loop)
                .unwrap(),
        );

        let renderer = Arc::new(Renderer::new());

        App {
            event_loop,
            window,
            renderer,
        }
    }

    pub fn context(&self) -> AppContex {
        AppContex {
            window: self.window.clone(),
            renderer: self.renderer.clone(),
        }
    }

    pub fn run(self, app_loop: impl FnMut(AppEvent<'_>) -> AppState + 'static) -> ! {
        use std::mem::ManuallyDrop;

        let App {
            event_loop,
            window,
            renderer,
        } = self;

        let mut active = true;

        let frame = Framework::new(
            window.clone(),
            renderer.clone(),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        );

        let mut window = ManuallyDrop::new(window);
        let mut renderer = ManuallyDrop::new(renderer);
        let mut app_loop = ManuallyDrop::new(app_loop);
        let mut frame = ManuallyDrop::new(frame);

        event_loop.run(move |event, _loop, control_flow| {
            match event {
                Event::NewEvents(_) => { /*app.on_new_events(),*/ }
                Event::WindowEvent { window_id, event } => {
                    frame.on_window_event(window_id, &event);

                    if window_id == window.id() {
                        let state = match event {
                            WindowEvent::CloseRequested => (app_loop)(AppEvent::CloseRequested),
                            _ => AppState::Run,
                        };

                        match state {
                            AppState::Run => *control_flow = ControlFlow::Poll,
                            AppState::Exit => *control_flow = ControlFlow::Exit,
                        }
                    }

                    // app.on_window_event(window_id, &event);
                }
                Event::DeviceEvent { device_id, event } => {
                    frame.on_device_event(device_id, &event);
                    // app.on_device_event(device_id, &event);
                }
                Event::Suspended => {
                    active = false;
                }
                Event::Resumed => {
                    active = true;
                }
                Event::MainEventsCleared => {
                    if active {
                        frame.begin_frame();
                        let state = (app_loop)(AppEvent::Frame {
                            ctx: frame.context(),
                            frame: &mut frame,
                        });

                        match state {
                            AppState::Run => *control_flow = ControlFlow::Poll,
                            AppState::Exit => *control_flow = ControlFlow::Exit,
                        }
                        frame.end_frame().unwrap();
                    }
                }
                Event::LoopDestroyed => {
                    // Safety: LoopDestroyed is the last event called before the function terminates
                    // so window, renderer, and app_loop will never be used again.
                    unsafe {
                        ManuallyDrop::drop(&mut app_loop);
                        ManuallyDrop::drop(&mut frame);
                        ManuallyDrop::drop(&mut window);
                        ManuallyDrop::drop(&mut renderer);
                    }
                }
                // other application-specific event handling
                _ => {
                    // platform.handle_event(imgui.io_mut(), &window, &event); // step 3
                    // other application-specific event handling
                }
            }
        });
    }
}

fn destruct_app(app: App) -> (EventLoop<()>, Arc<Window>, Arc<Renderer>) {
    let App {
        event_loop,
        window,
        renderer,
    } = app;
    (event_loop, window, renderer)
}
