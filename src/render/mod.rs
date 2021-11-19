use wgpu::{
    Adapter, Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    Instance, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};

use log::info;

use winit::event::WindowEvent;
use winit::window::{Window, WindowId};

use std::borrow::Borrow;
use std::sync::Arc;

pub struct Renderer {
    window: Arc<Window>,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        futures::executor::block_on(async {
            info!("Initializing Renderer");

            info!("Initializing Instance");
            let instance = Instance::new(Backends::all());
            info!("Initializing Surface");
            let surface = unsafe { instance.create_surface::<Window>(window.borrow()) };
            info!("Initializing Adapter");
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    power_preference: PowerPreference::HighPerformance,
                })
                .await
                .unwrap();
            info!("Initializing Device");
            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        label: Some("Rendering GPU"),
                        features: Features::empty(),
                        limits: Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();
            info!("Initializing Swapchain");
            surface.configure(
                &device,
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: surface.get_preferred_format(&adapter).unwrap(),
                    width: window.inner_size().width,
                    height: window.inner_size().height,
                    present_mode: PresentMode::Fifo,
                },
            );

            Self {
                window,
                surface,
                adapter,
                device,
                queue,
            }
        })
    }

    pub fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        if self.window.id() == id {
            match event {
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        self.surface.configure(
                            &self.device,
                            &SurfaceConfiguration {
                                usage: TextureUsages::RENDER_ATTACHMENT,
                                format: self.surface.get_preferred_format(&self.adapter).unwrap(),
                                width: size.width,
                                height: size.height,
                                present_mode: PresentMode::Fifo,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
    }

    pub fn on_redraw_event(&mut self, id: WindowId) {
        if self.window.id() == id {
            let output = self.surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            {
                let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
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
            }

            // submit will accept anything that implements IntoIter
            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}
