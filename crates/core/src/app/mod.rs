pub mod frame;

use crate::render::RenderCtxRef;
use egui::CtxRef;
use frame::Framework;
use std::borrow::Borrow;
use std::sync::Arc;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;

pub enum AppEvent {
    CloseRequested,
    Frame { ctx: CtxRef },
}

pub enum AppState {
    Run,
    Exit,
}

#[derive(Clone)]
pub struct AppContex {
    window: Arc<Window>,
    renderer: RenderCtxRef,
}

impl AppContex {
    pub fn window(&self) -> &Window {
        self.window.borrow()
    }

    pub fn render(&self) -> RenderCtxRef {
        self.renderer.clone()
    }
}

pub struct App {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    renderer: RenderCtxRef,
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

        let renderer = RenderCtxRef::new();

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

    pub fn run(self, app_loop: impl FnMut(AppEvent) -> AppState + 'static) -> ! {
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
            wgpu::TextureFormat::Bgra8Unorm,
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