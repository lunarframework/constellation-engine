pub mod clipboard;

use crate::render::RenderCtxRef;
use bytemuck::{Pod, Zeroable};
use std::borrow::Borrow;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::{
    util::DeviceExt, PresentMode, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::event::{DeviceEvent, DeviceId};
use winit::window::{Window, WindowId};

/// Builder of ui and manages the various user interations with that ui.
/// This is a thin wrapper atop `imgui`, an immediete mode gui lib.
pub struct Framework {
    window: Arc<Window>,

    // Egui
    context: egui::CtxRef,
    start_time: std::time::Instant,
    raw_input: egui::RawInput,
    pointer_pos_in_points: Option<egui::Pos2>,
    any_pointer_button_down: bool,
    current_cursor_icon: egui::CursorIcon,
    current_pixels_per_point: f32,
    pointer_touch_id: Option<u64>,
    /// If `true`, mouse inputs will be treated as touches.
    /// Useful for debugging touch support in egui.
    ///
    /// Creates duplicate touches, if real touch inputs are coming.
    simulate_touch_screen: bool,
    clipboard: clipboard::Clipboard,

    // Composite rendering
    renderer: RenderCtxRef,
    surface: Surface,
    surface_format: TextureFormat,
    render_pipeline: wgpu::RenderPipeline,
    index_buffers: Vec<SizedBuffer>,
    vertex_buffers: Vec<SizedBuffer>,
    uniform_buffer: SizedBuffer,
    uniform_bind_group: wgpu::BindGroup,
    font_texture_bind_group: Option<wgpu::BindGroup>,
    font_texture_version: Option<u64>,
}

impl Framework {
    /// Creates the gui handler and attaches it to the given window
    pub fn new(window: Arc<Window>, renderer: RenderCtxRef, format: TextureFormat) -> Self {
        let pixels_per_point = window.scale_factor() as f32;
        let context = egui::CtxRef::default();
        context.set_fonts(egui::FontDefinitions::default());
        context.set_style(egui::Style::default());

        use egui::epaint::Shadow;
        use egui::style::Selection;
        use egui::style::{WidgetVisuals, Widgets};
        use egui::{Color32, Stroke};

        let visuals = egui::Visuals {
            dark_mode: true,
            override_text_color: None,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_gray(27), // window background
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines, windows outlines
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                    corner_radius: 2.0,
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_gray(60), // button background
                    bg_stroke: Default::default(),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                    corner_radius: 2.0,
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_gray(70),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                    fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                    corner_radius: 3.0,
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: Color32::from_gray(55),
                    bg_stroke: Stroke::new(1.0, Color32::WHITE),
                    fg_stroke: Stroke::new(2.0, Color32::WHITE),
                    corner_radius: 2.0,
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: Color32::from_gray(27),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                    corner_radius: 2.0,
                    expansion: 0.0,
                },
            },
            selection: Selection::default(),
            hyperlink_color: Color32::from_rgb(90, 170, 255),
            faint_bg_color: Color32::from_gray(24),
            extreme_bg_color: Color32::from_gray(10),
            code_bg_color: Color32::from_gray(64),
            window_corner_radius: 8.0,
            window_shadow: Shadow::big_dark(),
            popup_shadow: Shadow::small_dark(),
            resize_corner_size: 12.0,
            text_cursor_width: 2.0,
            text_cursor_preview: false,
            clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
            button_frame: true,
            collapsing_header_frame: false,
        };
        context.set_visuals(visuals);

        // SAFTEY: Self holds a reference to a window, ensuring it lives just as long as the surface.
        let surface = unsafe {
            renderer
                .instance()
                .create_surface::<Window>(window.borrow())
        };

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
        let module = renderer
            .device()
            .create_shader_module(&wgpu::include_wgsl!("composite.wgsl"));
        let uniform_buffer =
            renderer
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("composite_uniform_buffer"),
                    contents: bytemuck::cast_slice(&[UniformBuffer {
                        screen_size: [0.0, 0.0],
                    }]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let uniform_buffer = SizedBuffer {
            buffer: uniform_buffer,
            size: std::mem::size_of::<UniformBuffer>(),
        };

        let uniform_bind_group_layout =
            renderer
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("composite_uniform_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Uniform,
                        },
                        count: None,
                    }],
                });

        let uniform_bind_group = renderer
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite_uniform_bind_group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer.buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let pipeline_layout =
            renderer
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("composite_pipeline_layout"),
                    bind_group_layouts: &[
                        &uniform_bind_group_layout,
                        &renderer.image_bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = renderer.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("composite_pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        entry_point: "vs_main",
                        module: &module,
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: 5 * 4,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            // 0: vec2 position
                            // 1: vec2 texture coordinates
                            // 2: uint color
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32],
                }],
            },
            multiview: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                unclipped_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: wgpu::FrontFace::default(),
                polygon_mode: wgpu::PolygonMode::default(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
        });
        Self {
            window,
            renderer,
            surface,
            surface_format: format,

            context,
            start_time: std::time::Instant::now(),
            raw_input: egui::RawInput {
                pixels_per_point: Some(pixels_per_point),
                ..Default::default()
            },
            pointer_pos_in_points: None,
            any_pointer_button_down: false,
            current_cursor_icon: egui::CursorIcon::Default,
            current_pixels_per_point: pixels_per_point,
            pointer_touch_id: None,
            simulate_touch_screen: false,
            clipboard: Default::default(),
            render_pipeline,
            vertex_buffers: Vec::with_capacity(64),
            index_buffers: Vec::with_capacity(64),
            uniform_buffer,
            uniform_bind_group,
            font_texture_version: None,
            font_texture_bind_group: None,
        }
    }
    /// The number of physical pixels per logical point,
    /// as configured on the current egui context (see [`egui::Context::pixels_per_point`]).
    #[inline]
    pub fn pixels_per_point(&self) -> f32 {
        self.current_pixels_per_point
    }
    /// The current input state.
    /// This is changed by [`Self::on_event`] and cleared by [`Self::take_egui_input`].
    #[inline]
    pub fn raw_input(&self) -> &egui::RawInput {
        &self.raw_input
    }
    /// Returns the imgui context
    pub fn context(&self) -> egui::CtxRef {
        self.context.clone()
    }
    pub fn on_window_event(&mut self, id: WindowId, event: &winit::event::WindowEvent) {
        if self.window.id() == id {
            use winit::event::WindowEvent;
            match event {
                WindowEvent::Resized(size) => {
                    // TODO, why does winit seem to throw an invalid resize event
                    if size.width > 0 && size.height > 0 {
                        let size = self.window.inner_size();
                        // let format = self
                        //     .surface
                        //     .get_preferred_format(self.renderer.adapter())
                        //     .unwrap();
                        self.surface.configure(
                            self.renderer.device(),
                            &SurfaceConfiguration {
                                usage: TextureUsages::RENDER_ATTACHMENT,
                                format: self.surface_format,
                                width: size.width,
                                height: size.height,
                                present_mode: PresentMode::Fifo,
                            },
                        );
                        // self.format = format;
                    }
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let pixels_per_point = *scale_factor as f32;
                    self.raw_input.pixels_per_point = Some(pixels_per_point);
                    self.current_pixels_per_point = pixels_per_point;
                    // false
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    self.on_mouse_button_input(*state, *button);
                    // egui_ctx.wants_pointer_input()
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    self.on_mouse_wheel(*delta);
                    // egui_ctx.wants_pointer_input()
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.on_cursor_moved(*position);
                    // egui_ctx.is_using_pointer()
                }
                WindowEvent::CursorLeft { .. } => {
                    self.pointer_pos_in_points = None;
                    self.raw_input.events.push(egui::Event::PointerGone);
                    // false
                }
                // WindowEvent::TouchpadPressure {device_id, pressure, stage, ..  } => {} // TODO
                WindowEvent::Touch(touch) => {
                    self.on_touch(touch);
                    match touch.phase {
                        winit::event::TouchPhase::Started
                        | winit::event::TouchPhase::Ended
                        | winit::event::TouchPhase::Cancelled => self.context.wants_pointer_input(),
                        winit::event::TouchPhase::Moved => self.context.is_using_pointer(),
                    };
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    // On Mac we get here when the user presses Cmd-C (copy), ctrl-W, etc.
                    // We need to ignore these characters that are side-effects of commands.
                    let is_mac_cmd = cfg!(target_os = "macos")
                        && (self.raw_input.modifiers.ctrl || self.raw_input.modifiers.mac_cmd);

                    if is_printable_char(*ch) && !is_mac_cmd {
                        self.raw_input
                            .events
                            .push(egui::Event::Text(ch.to_string()));
                        //self.context.wants_keyboard_input()
                    } else {
                        //false
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    self.on_keyboard_input(input);
                    // egui_ctx.wants_keyboard_input() || input.virtual_keycode == Some(winit::event::VirtualKeyCode::Tab)
                }
                WindowEvent::Focused(_) => {
                    // We will not be given a KeyboardInput event when the modifiers are released while
                    // the window does not have focus. Unset all modifier state to be safe.
                    self.raw_input.modifiers = egui::Modifiers::default();
                    // false
                }
                WindowEvent::HoveredFile(path) => {
                    self.raw_input.hovered_files.push(egui::HoveredFile {
                        path: Some(path.clone()),
                        ..Default::default()
                    });
                    // false
                }
                WindowEvent::HoveredFileCancelled => {
                    self.raw_input.hovered_files.clear();
                    // false
                }
                WindowEvent::DroppedFile(path) => {
                    self.raw_input.hovered_files.clear();
                    self.raw_input.dropped_files.push(egui::DroppedFile {
                        path: Some(path.clone()),
                        ..Default::default()
                    });
                    // false
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.raw_input.modifiers.alt = state.alt();
                    self.raw_input.modifiers.ctrl = state.ctrl();
                    self.raw_input.modifiers.shift = state.shift();
                    self.raw_input.modifiers.mac_cmd = cfg!(target_os = "macos") && state.logo();
                    self.raw_input.modifiers.command = if cfg!(target_os = "macos") {
                        state.logo()
                    } else {
                        state.ctrl()
                    };
                    // false
                }
                _ => {
                    // dbg!(event);
                    // false
                }
            }
        }
    }

    fn on_mouse_button_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        if let Some(pos) = self.pointer_pos_in_points {
            if let Some(button) = translate_mouse_button(button) {
                let pressed = state == winit::event::ElementState::Pressed;

                self.raw_input.events.push(egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: self.raw_input.modifiers,
                });

                if self.simulate_touch_screen {
                    if pressed {
                        self.any_pointer_button_down = true;

                        self.raw_input.events.push(egui::Event::Touch {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::Start,
                            pos,
                            force: 0.0,
                        });
                    } else {
                        self.any_pointer_button_down = false;

                        self.raw_input.events.push(egui::Event::PointerGone);

                        self.raw_input.events.push(egui::Event::Touch {
                            device_id: egui::TouchDeviceId(0),
                            id: egui::TouchId(0),
                            phase: egui::TouchPhase::End,
                            pos,
                            force: 0.0,
                        });
                    };
                }
            }
        }
    }

    fn on_cursor_moved(&mut self, pos_in_pixels: winit::dpi::PhysicalPosition<f64>) {
        let pos_in_points = egui::pos2(
            pos_in_pixels.x as f32 / self.pixels_per_point(),
            pos_in_pixels.y as f32 / self.pixels_per_point(),
        );
        self.pointer_pos_in_points = Some(pos_in_points);

        if self.simulate_touch_screen {
            if self.any_pointer_button_down {
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(pos_in_points));

                self.raw_input.events.push(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(0),
                    phase: egui::TouchPhase::Move,
                    pos: pos_in_points,
                    force: 0.0,
                });
            }
        } else {
            self.raw_input
                .events
                .push(egui::Event::PointerMoved(pos_in_points));
        }
    }

    fn on_touch(&mut self, touch: &winit::event::Touch) {
        // Emit touch event
        self.raw_input.events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(egui::epaint::util::hash(touch.device_id)),
            id: egui::TouchId::from(touch.id),
            phase: match touch.phase {
                winit::event::TouchPhase::Started => egui::TouchPhase::Start,
                winit::event::TouchPhase::Moved => egui::TouchPhase::Move,
                winit::event::TouchPhase::Ended => egui::TouchPhase::End,
                winit::event::TouchPhase::Cancelled => egui::TouchPhase::Cancel,
            },
            pos: egui::pos2(
                touch.location.x as f32 / self.pixels_per_point(),
                touch.location.y as f32 / self.pixels_per_point(),
            ),
            force: match touch.force {
                Some(winit::event::Force::Normalized(force)) => force as f32,
                Some(winit::event::Force::Calibrated {
                    force,
                    max_possible_force,
                    ..
                }) => (force / max_possible_force) as f32,
                None => 0_f32,
            },
        });
        // If we're not yet tanslating a touch or we're translating this very
        // touch …
        if self.pointer_touch_id.is_none() || self.pointer_touch_id.unwrap() == touch.id {
            // … emit PointerButton resp. PointerMoved events to emulate mouse
            match touch.phase {
                winit::event::TouchPhase::Started => {
                    self.pointer_touch_id = Some(touch.id);
                    // First move the pointer to the right location
                    self.on_cursor_moved(touch.location);
                    self.on_mouse_button_input(
                        winit::event::ElementState::Pressed,
                        winit::event::MouseButton::Left,
                    );
                }
                winit::event::TouchPhase::Moved => {
                    self.on_cursor_moved(touch.location);
                }
                winit::event::TouchPhase::Ended => {
                    self.pointer_touch_id = None;
                    self.on_mouse_button_input(
                        winit::event::ElementState::Released,
                        winit::event::MouseButton::Left,
                    );
                    // The pointer should vanish completely to not get any
                    // hover effects
                    self.pointer_pos_in_points = None;
                    self.raw_input.events.push(egui::Event::PointerGone);
                }
                winit::event::TouchPhase::Cancelled => {
                    self.pointer_touch_id = None;
                    self.pointer_pos_in_points = None;
                    self.raw_input.events.push(egui::Event::PointerGone);
                }
            }
        }
    }

    fn on_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        let mut delta = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                let points_per_scroll_line = 50.0; // Scroll speed decided by consensus: https://github.com/emilk/egui/issues/461
                egui::vec2(x, y) * points_per_scroll_line
            }
            winit::event::MouseScrollDelta::PixelDelta(delta) => {
                egui::vec2(delta.x as f32, delta.y as f32) / self.pixels_per_point()
            }
        };
        if cfg!(target_os = "macos") {
            // This is still buggy in winit despite
            // https://github.com/rust-windowing/winit/issues/1695 being closed
            delta.x *= -1.0;
        }

        if self.raw_input.modifiers.ctrl || self.raw_input.modifiers.command {
            // Treat as zoom instead:
            let factor = (delta.y / 200.0).exp();
            self.raw_input.events.push(egui::Event::Zoom(factor));
        } else {
            self.raw_input.events.push(egui::Event::Scroll(delta));
        }
    }

    fn on_keyboard_input(&mut self, input: &winit::event::KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            let pressed = input.state == winit::event::ElementState::Pressed;

            if pressed {
                // VirtualKeyCode::Paste etc in winit are broken/untrustworthy,
                // so we detect these things manually:
                if is_cut_command(self.raw_input.modifiers, keycode) {
                    self.raw_input.events.push(egui::Event::Cut);
                } else if is_copy_command(self.raw_input.modifiers, keycode) {
                    self.raw_input.events.push(egui::Event::Copy);
                } else if is_paste_command(self.raw_input.modifiers, keycode) {
                    if let Some(contents) = self.clipboard.get() {
                        self.raw_input
                            .events
                            .push(egui::Event::Text(contents.replace("\r\n", "\n")));
                    }
                }
            }

            if let Some(key) = translate_virtual_key_code(keycode) {
                self.raw_input.events.push(egui::Event::Key {
                    key,
                    pressed,
                    modifiers: self.raw_input.modifiers,
                });
            }
        }
    }

    pub fn on_device_event(&mut self, _id: DeviceId, event: &DeviceEvent) {
        match event {
            _ => {}
        }
    }

    /// Updates the internal time for egui used for animations. `elapsed_seconds` should be the seconds since some point in time (for example application start).
    pub fn update_time(&mut self, elapsed_seconds: f64) {
        self.raw_input.time = Some(elapsed_seconds);
    }

    /// Starts a new frame by providing a new `Ui` instance to write into.
    pub fn begin_frame(&mut self) {
        let pixels_per_point = self.pixels_per_point();

        self.raw_input.time = Some(self.start_time.elapsed().as_secs_f64());

        // On Windows, a minimized window will have 0 width and height.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where egui window positions would be changed when minimizing on Windows.
        let screen_size_in_pixels = screen_size_in_pixels(self.window.borrow());
        let screen_size_in_points = screen_size_in_pixels / pixels_per_point;
        self.raw_input.screen_rect =
            if screen_size_in_points.x > 0.0 && screen_size_in_points.y > 0.0 {
                Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    screen_size_in_points,
                ))
            } else {
                None
            };
        self.context.begin_frame(self.raw_input.take());
    }

    /// Ends the frame.
    pub fn end_frame(&mut self) -> Result<(), FrameError> {
        self.current_pixels_per_point = self.pixels_per_point(); // someone can have changed it to scale the UI

        let (output, shapes) = self.context().end_frame();

        // if egui_ctx.memory().options.screen_reader {
        //     self.screen_reader.speak(&output.events_description());
        // }

        // if let Some(open) = output.open_url {
        //     open_url(&open.url);
        // }

        if !output.copied_text.is_empty() {
            self.clipboard.set(output.copied_text);
        }

        self.set_cursor_icon(output.cursor_icon);

        if let Some(egui::Pos2 { x, y }) = output.text_cursor_pos {
            self.window
                .set_ime_position(winit::dpi::LogicalPosition { x, y });
        }

        let paint_jobs = self.context().tessellate(shapes);

        self.update_font_image();
        self.update_buffers(&paint_jobs);

        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                return Err(FrameError::Internal(format!(
                    "Dropped frame with error: {}",
                    e
                )));
            }
        };
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Composite Command Encoder"),
                });

        let images = self.renderer.get_image_bind_groups().lock().unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let scale_factor = self.window.scale_factor() as f32;
        let physical_width = self.window.inner_size().width;
        let physical_height = self.window.inner_size().height;

        for ((egui::ClippedMesh(clip_rect, mesh), vertex_buffer), index_buffer) in paint_jobs
            .iter()
            .zip(self.vertex_buffers.iter())
            .zip(self.index_buffers.iter())
        {
            // Transform clip rect to physical pixels.
            let clip_min_x = scale_factor * clip_rect.min.x;
            let clip_min_y = scale_factor * clip_rect.min.y;
            let clip_max_x = scale_factor * clip_rect.max.x;
            let clip_max_y = scale_factor * clip_rect.max.y;

            // Make sure clip rect can fit within an `u32`.
            let clip_min_x = clip_min_x.clamp(0.0, physical_width as f32);
            let clip_min_y = clip_min_y.clamp(0.0, physical_height as f32);
            let clip_max_x = clip_max_x.clamp(clip_min_x, physical_width as f32);
            let clip_max_y = clip_max_y.clamp(clip_min_y, physical_height as f32);

            let clip_min_x = clip_min_x.round() as u32;
            let clip_min_y = clip_min_y.round() as u32;
            let clip_max_x = clip_max_x.round() as u32;
            let clip_max_y = clip_max_y.round() as u32;

            let width = (clip_max_x - clip_min_x).max(1);
            let height = (clip_max_y - clip_min_y).max(1);

            {
                // Clip scissor rectangle to target size.
                let x = clip_min_x.min(physical_width);
                let y = clip_min_y.min(physical_height);
                let width = width.min(physical_width - x);
                let height = height.min(physical_height - y);

                // Skip rendering with zero-sized clip areas.
                if width == 0 || height == 0 {
                    continue;
                }

                render_pass.set_scissor_rect(x, y, width, height);
            }

            match mesh.texture_id {
                egui::TextureId::Egui => {
                    render_pass.set_bind_group(
                        1,
                        (&self.font_texture_bind_group).as_ref().unwrap(),
                        &[],
                    );
                }
                egui::TextureId::User(id) => {
                    render_pass.set_bind_group(1, images.get(&id).unwrap(), &[]);
                }
            };

            render_pass.set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
            render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }

        drop(render_pass);
        drop(images);

        // submit will accept anything that implements IntoIter
        self.renderer
            .queue()
            .submit(std::iter::once(encoder.finish()));

        output_frame.present();

        Ok(())
    }

    fn set_cursor_icon(&mut self, cursor_icon: egui::CursorIcon) {
        // prevent flickering near frame boundary when Windows OS tries to control cursor icon for window resizing
        if self.current_cursor_icon == cursor_icon {
            return;
        }
        self.current_cursor_icon = cursor_icon;

        if let Some(cursor_icon) = translate_cursor(cursor_icon) {
            self.window.set_cursor_visible(true);

            let is_pointer_in_window = self.pointer_pos_in_points.is_some();
            if is_pointer_in_window {
                self.window.set_cursor_icon(cursor_icon);
            }
        } else {
            self.window.set_cursor_visible(false);
        }
    }

    // fn get_texture_bind_group(
    //     &self,
    //     texture_id: egui::TextureId,
    // ) -> Result<&wgpu::BindGroup, FrameError> {
    //     let bind_group = match texture_id {
    //         egui::TextureId::Egui => self.texture_bind_group.as_ref().ok_or_else(|| {
    //             FrameError::Internal("egui texture was not set before the first draw".to_string())
    //         })?,
    //         egui::TextureId::User(id) => {
    //             &(self.user_textures.get(&id).ok_or_else(|| {
    //                 FrameError::Internal(format!("user texture {} not found", id))
    //             })?)
    //         }
    //     };

    //     Ok(bind_group)
    // }

    fn update_font_image(&mut self) {
        let font_texture = self.context().font_image();
        // Don't update the texture if it hasn't changed.
        if self.font_texture_version == Some(font_texture.version) {
            return;
        }
        // we need to convert the texture into rgba_srgb format
        let mut pixels: Vec<u8> = Vec::with_capacity(font_texture.pixels.len() * 4);
        for srgba in font_texture.srgba_pixels(1.0) {
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }
        let egui_texture = egui::FontImage {
            version: font_texture.version,
            width: font_texture.width,
            height: font_texture.height,
            pixels,
        };
        let bind_group = self.font_image_to_wgpu(&egui_texture, "font");

        self.font_texture_version = Some(egui_texture.version);
        self.font_texture_bind_group = Some(bind_group);
    }

    /// Assumes egui_texture contains srgb data.
    /// This does not match how `egui::Texture` is documented as of writing, but this is how it is used for user textures.
    fn font_image_to_wgpu(&self, font_image: &egui::FontImage, label: &str) -> wgpu::BindGroup {
        let size = wgpu::Extent3d {
            width: font_image.width as u32,
            height: font_image.height as u32,
            depth_or_array_layers: 1,
        };

        let texture = self
            .renderer
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(format!("{}", label).as_str()),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            });

        self.renderer.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            font_image.pixels.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    (font_image.pixels.len() / font_image.height) as u32,
                ),
                rows_per_image: NonZeroU32::new(font_image.height as u32),
            },
            size,
        );

        let sampler = self
            .renderer
            .device()
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some(format!("{}_sampler", label).as_str()),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

        let bind_group = self
            .renderer
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(format!("{}_bind_group", label).as_str()),
                layout: &self.renderer.image_bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        bind_group
    }

    /// Uploads the uniform, vertex and index data used by the render pass.
    /// Should be called before `execute()`.
    fn update_buffers(&mut self, paint_jobs: &[egui::epaint::ClippedMesh]) {
        let index_size = self.index_buffers.len();
        let vertex_size = self.vertex_buffers.len();

        let logical = self
            .window
            .inner_size()
            .to_logical::<f32>(self.window.scale_factor());

        self.renderer.queue().write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[UniformBuffer {
                screen_size: [logical.width, logical.height],
            }]),
        );

        for (i, egui::ClippedMesh(_, mesh)) in paint_jobs.iter().enumerate() {
            let data: &[u8] = bytemuck::cast_slice(&mesh.indices);

            if i < index_size && data.len() <= self.index_buffers[i].size {
                self.renderer
                    .queue()
                    .write_buffer(&self.index_buffers[i].buffer, 0, data);
            } else {
                let buffer =
                    self.renderer
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("composite_index_buffer"),
                            contents: data,
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });

                if i < index_size {
                    self.index_buffers[i].buffer = buffer;
                    self.index_buffers[i].size = data.len();
                } else {
                    self.index_buffers.push(SizedBuffer {
                        buffer,
                        size: data.len(),
                    });
                }
            }

            let data: &[u8] = as_byte_slice(&mesh.vertices);

            if i < vertex_size && data.len() <= self.vertex_buffers[i].size {
                self.renderer
                    .queue()
                    .write_buffer(&self.vertex_buffers[i].buffer, 0, data);
            } else {
                let buffer =
                    self.renderer
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("composite_vertex_buffer"),
                            contents: data,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });

                if i < index_size {
                    self.vertex_buffers[i].buffer = buffer;
                    self.vertex_buffers[i].size = data.len();
                } else {
                    self.vertex_buffers.push(SizedBuffer {
                        buffer,
                        size: data.len(),
                    });
                }
            }
        }
    }
}

/// Winit sends special keys (backspace, delete, F1, …) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn is_cut_command(modifiers: egui::Modifiers, keycode: winit::event::VirtualKeyCode) -> bool {
    (modifiers.command && keycode == winit::event::VirtualKeyCode::X)
        || (cfg!(target_os = "windows")
            && modifiers.shift
            && keycode == winit::event::VirtualKeyCode::Delete)
}

fn is_copy_command(modifiers: egui::Modifiers, keycode: winit::event::VirtualKeyCode) -> bool {
    (modifiers.command && keycode == winit::event::VirtualKeyCode::C)
        || (cfg!(target_os = "windows")
            && modifiers.ctrl
            && keycode == winit::event::VirtualKeyCode::Insert)
}

fn is_paste_command(modifiers: egui::Modifiers, keycode: winit::event::VirtualKeyCode) -> bool {
    (modifiers.command && keycode == winit::event::VirtualKeyCode::V)
        || (cfg!(target_os = "windows")
            && modifiers.shift
            && keycode == winit::event::VirtualKeyCode::Insert)
}

fn translate_mouse_button(button: winit::event::MouseButton) -> Option<egui::PointerButton> {
    match button {
        winit::event::MouseButton::Left => Some(egui::PointerButton::Primary),
        winit::event::MouseButton::Right => Some(egui::PointerButton::Secondary),
        winit::event::MouseButton::Middle => Some(egui::PointerButton::Middle),
        winit::event::MouseButton::Other(_) => None,
    }
}

fn translate_virtual_key_code(key: winit::event::VirtualKeyCode) -> Option<egui::Key> {
    use egui::Key;
    use winit::event::VirtualKeyCode;

    Some(match key {
        VirtualKeyCode::Down => Key::ArrowDown,
        VirtualKeyCode::Left => Key::ArrowLeft,
        VirtualKeyCode::Right => Key::ArrowRight,
        VirtualKeyCode::Up => Key::ArrowUp,

        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Return => Key::Enter,
        VirtualKeyCode::Space => Key::Space,

        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::PageDown => Key::PageDown,

        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Key::Num0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Key::Num1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Key::Num2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Key::Num3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Key::Num4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Key::Num5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Key::Num6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Key::Num7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Key::Num8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Key::Num9,

        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,

        _ => {
            return None;
        }
    })
}

fn translate_cursor(cursor_icon: egui::CursorIcon) -> Option<winit::window::CursorIcon> {
    match cursor_icon {
        egui::CursorIcon::None => None,

        egui::CursorIcon::Alias => Some(winit::window::CursorIcon::Alias),
        egui::CursorIcon::AllScroll => Some(winit::window::CursorIcon::AllScroll),
        egui::CursorIcon::Cell => Some(winit::window::CursorIcon::Cell),
        egui::CursorIcon::ContextMenu => Some(winit::window::CursorIcon::ContextMenu),
        egui::CursorIcon::Copy => Some(winit::window::CursorIcon::Copy),
        egui::CursorIcon::Crosshair => Some(winit::window::CursorIcon::Crosshair),
        egui::CursorIcon::Default => Some(winit::window::CursorIcon::Default),
        egui::CursorIcon::Grab => Some(winit::window::CursorIcon::Grab),
        egui::CursorIcon::Grabbing => Some(winit::window::CursorIcon::Grabbing),
        egui::CursorIcon::Help => Some(winit::window::CursorIcon::Help),
        egui::CursorIcon::Move => Some(winit::window::CursorIcon::Move),
        egui::CursorIcon::NoDrop => Some(winit::window::CursorIcon::NoDrop),
        egui::CursorIcon::NotAllowed => Some(winit::window::CursorIcon::NotAllowed),
        egui::CursorIcon::PointingHand => Some(winit::window::CursorIcon::Hand),
        egui::CursorIcon::Progress => Some(winit::window::CursorIcon::Progress),
        egui::CursorIcon::ResizeHorizontal => Some(winit::window::CursorIcon::EwResize),
        egui::CursorIcon::ResizeNeSw => Some(winit::window::CursorIcon::NeswResize),
        egui::CursorIcon::ResizeNwSe => Some(winit::window::CursorIcon::NwseResize),
        egui::CursorIcon::ResizeVertical => Some(winit::window::CursorIcon::NsResize),
        egui::CursorIcon::Text => Some(winit::window::CursorIcon::Text),
        egui::CursorIcon::VerticalText => Some(winit::window::CursorIcon::VerticalText),
        egui::CursorIcon::Wait => Some(winit::window::CursorIcon::Wait),
        egui::CursorIcon::ZoomIn => Some(winit::window::CursorIcon::ZoomIn),
        egui::CursorIcon::ZoomOut => Some(winit::window::CursorIcon::ZoomOut),
    }
}

fn screen_size_in_pixels(window: &winit::window::Window) -> egui::Vec2 {
    let size = window.inner_size();
    egui::vec2(size.width as f32, size.height as f32)
}

/// Error that the backend can return.
#[derive(Debug)]
pub enum FrameError {
    /// The given `egui::TextureId` was invalid.
    InvalidTextureId(String),
    /// Internal implementation error.
    Internal(String),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::InvalidTextureId(msg) => {
                write!(f, "invalid TextureId: `{:?}`", msg)
            }
            FrameError::Internal(msg) => {
                write!(f, "internal error: `{:?}`", msg)
            }
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            _ => None,
        }
    }
}

/// Uniform buffer used when rendering.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct UniformBuffer {
    screen_size: [f32; 2],
}

unsafe impl Pod for UniformBuffer {}

unsafe impl Zeroable for UniformBuffer {}

/// Wraps the buffers and includes additional information.
#[derive(Debug)]
struct SizedBuffer {
    buffer: wgpu::Buffer,
    size: usize,
}

// Needed since we can't use bytemuck for external types.
fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
    let len = slice.len() * std::mem::size_of::<T>();
    let ptr = slice.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}
