use clap::ArgMatches;

use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;

use std::borrow::Borrow;
use std::sync::Arc;

pub fn test(_matches: &ArgMatches) {
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Constellation Engine")
            .with_inner_size(winit::dpi::PhysicalSize {
                width: 800,
                height: 600,
            })
            .build(&event_loop)
            .unwrap(),
    );

    let instance = wgpu::Instance::new(wgpu::Backends::DX12);
    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: None,
                force_fallback_adapter: false,
                power_preference: wgpu::PowerPreference::LowPower,
            })
            .await
            .unwrap();

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap()
    });

    let surface = unsafe { instance.create_surface::<Window>(window.borrow()) };

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
        },
    );

    event_loop.run(move |event, _loop, control_flow| {
        match event {
            Event::NewEvents(_) => { /*app.on_new_events(),*/ }
            Event::WindowEvent { window_id, event } => {
                if window_id == window.id() {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    }
                }

                // app.on_window_event(window_id, &event);
            }
            Event::MainEventsCleared => {
                let output_frame = surface.get_current_texture().unwrap();
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Composite Command Encoder"),
                });

                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Paint Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                drop(render_pass);

                // submit will accept anything that implements IntoIter
                queue.submit(std::iter::once(encoder.finish()));

                let dummy_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Dummy Command Encoder"),
                    });

                queue.submit(std::iter::once(dummy_encoder.finish()));

                output_frame.present();
            }
            // other application-specific event handling
            _ => {
                // platform.handle_event(imgui.io_mut(), &window, &event); // step 3
                // other application-specific event handling
            }
        }
    });
}
