use log::info;
use wgpu::{PresentMode, Surface, SurfaceConfiguration, Texture, TextureFormat, TextureUsages};
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};

use std::borrow::Borrow;
use std::sync::Arc;

use super::Renderer;

pub struct Canvas {
    window: Arc<Window>,
    renderer: Arc<Renderer>,
    surface: Surface,
    format: TextureFormat,
}

impl Canvas {
    pub fn new(window: Arc<Window>, renderer: Arc<Renderer>) -> Self {
        info!("Initializing Canvas");
        let surface = unsafe {
            renderer
                .instance()
                .create_surface::<Window>(window.borrow())
        };

        let format = surface.get_preferred_format(renderer.adapter()).unwrap();

        surface.configure(
            renderer.device(),
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: format,
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: PresentMode::Fifo,
            },
        );

        Self {
            window,
            renderer,
            surface,
            format,
        }
    }

    pub fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        if self.window.id() == id {
            match event {
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        let format = self
                            .surface
                            .get_preferred_format(self.renderer.adapter())
                            .unwrap();

                        self.surface.configure(
                            self.renderer.device(),
                            &SurfaceConfiguration {
                                usage: TextureUsages::RENDER_ATTACHMENT,
                                format: format,
                                width: size.width,
                                height: size.height,
                                present_mode: PresentMode::Fifo,
                            },
                        );

                        self.format = format;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn present<F: FnOnce(&Texture) -> bool>(&self, f: F) {
        let output = self.surface.get_current_texture().unwrap();
        let touched = f(&output.texture);
        if touched {
            output.present();
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }
}