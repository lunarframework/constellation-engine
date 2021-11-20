use imgui::{BackendFlags, ConfigFlags, Context, FontAtlasRefMut, Io, Key, Ui};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase,
    VirtualKeyCode, WindowEvent,
};
use winit::window::{CursorIcon, Window, WindowId};

use std::cmp::Ordering;
use std::sync::Arc;

use super::{ImageDescriptor, ImageId, Painter};

/// Controls how different dpi levels are handled in the ui.
#[derive(Clone, Debug, PartialEq)]
pub enum HiDpiMode {
    Default,
    Rounded,
    Locked(f64),
}

#[derive(Clone)]
enum ActiveHiDpiMode {
    Default,
    Rounded,
    Locked,
}

#[derive(Clone)]
struct DpiHandler {
    mode: ActiveHiDpiMode,
    factor: f64,
}

impl DpiHandler {
    fn new(mode: HiDpiMode, factor: f64) -> Self {
        match mode {
            HiDpiMode::Default => Self {
                mode: ActiveHiDpiMode::Default,
                factor,
            },
            HiDpiMode::Rounded => Self {
                mode: ActiveHiDpiMode::Rounded,
                factor: factor.round(),
            },
            HiDpiMode::Locked(value) => Self {
                mode: ActiveHiDpiMode::Locked,
                factor: value,
            },
        }
    }

    fn adjust_logical_size(
        &self,
        window: &Window,
        logical_size: LogicalSize<f64>,
    ) -> LogicalSize<f64> {
        match self.mode {
            ActiveHiDpiMode::Default => logical_size,
            _ => logical_size
                .to_physical::<f64>(window.scale_factor())
                .to_logical(self.factor),
        }
    }

    fn adjust_logical_pos(
        &self,
        window: &Window,
        logical_pos: LogicalPosition<f64>,
    ) -> LogicalPosition<f64> {
        match self.mode {
            ActiveHiDpiMode::Default => logical_pos,
            _ => logical_pos
                .to_physical::<f64>(window.scale_factor())
                .to_logical(self.factor),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct CursorSettings {
    cursor: Option<imgui::MouseCursor>,
    draw_cursor: bool,
}

fn to_winit_cursor(cursor: imgui::MouseCursor) -> CursorIcon {
    match cursor {
        imgui::MouseCursor::Arrow => CursorIcon::Default,
        imgui::MouseCursor::TextInput => CursorIcon::Text,
        imgui::MouseCursor::ResizeAll => CursorIcon::Move,
        imgui::MouseCursor::ResizeNS => CursorIcon::NsResize,
        imgui::MouseCursor::ResizeEW => CursorIcon::EwResize,
        imgui::MouseCursor::ResizeNESW => CursorIcon::NeswResize,
        imgui::MouseCursor::ResizeNWSE => CursorIcon::NwseResize,
        imgui::MouseCursor::Hand => CursorIcon::Hand,
        imgui::MouseCursor::NotAllowed => CursorIcon::NotAllowed,
    }
}

impl CursorSettings {
    fn apply(&self, window: &Window) {
        match self.cursor {
            Some(mouse_cursor) if !self.draw_cursor => {
                window.set_cursor_visible(true);
                window.set_cursor_icon(to_winit_cursor(mouse_cursor));
            }
            _ => window.set_cursor_visible(false),
        }
    }
}

/// Builder of ui and manages the various user interations with that ui.
/// This is a thin wrapper atop `imgui`, an immediete mode gui lib.
pub struct UiManager {
    window: Arc<Window>,
    context: Context,
    dpi: DpiHandler,
    cursor_cache: Option<CursorSettings>,
    font_atlas_image: Option<ImageId>,
}

impl UiManager {
    /// Creates the gui handler and attaches it to the given window
    pub fn new(window: Arc<Window>, mode: HiDpiMode) -> Self {
        let mut context = Context::create();

        context.set_platform_name(Some(format!(
            "constellation-engine {}",
            env!("CARGO_PKG_VERSION")
        )));

        let dpi = DpiHandler::new(mode, window.scale_factor());

        {
            let io = context.io_mut();
            io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
            io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
            io[Key::Tab] = VirtualKeyCode::Tab as _;
            io[Key::LeftArrow] = VirtualKeyCode::Left as _;
            io[Key::RightArrow] = VirtualKeyCode::Right as _;
            io[Key::UpArrow] = VirtualKeyCode::Up as _;
            io[Key::DownArrow] = VirtualKeyCode::Down as _;
            io[Key::PageUp] = VirtualKeyCode::PageUp as _;
            io[Key::PageDown] = VirtualKeyCode::PageDown as _;
            io[Key::Home] = VirtualKeyCode::Home as _;
            io[Key::End] = VirtualKeyCode::End as _;
            io[Key::Insert] = VirtualKeyCode::Insert as _;
            io[Key::Delete] = VirtualKeyCode::Delete as _;
            io[Key::Backspace] = VirtualKeyCode::Back as _;
            io[Key::Space] = VirtualKeyCode::Space as _;
            io[Key::Enter] = VirtualKeyCode::Return as _;
            io[Key::Escape] = VirtualKeyCode::Escape as _;
            io[Key::KeyPadEnter] = VirtualKeyCode::NumpadEnter as _;
            io[Key::A] = VirtualKeyCode::A as _;
            io[Key::C] = VirtualKeyCode::C as _;
            io[Key::V] = VirtualKeyCode::V as _;
            io[Key::X] = VirtualKeyCode::X as _;
            io[Key::Y] = VirtualKeyCode::Y as _;
            io[Key::Z] = VirtualKeyCode::Z as _;

            io.display_framebuffer_scale = [dpi.factor as f32, dpi.factor as f32];
            let logical_size = window.inner_size().to_logical(dpi.factor);
            let logical_size = dpi.adjust_logical_size(&window, logical_size);
            io.display_size = [logical_size.width as f32, logical_size.height as f32];
        }

        Self {
            context,
            dpi,
            window,
            cursor_cache: Option::<CursorSettings>::None,
            font_atlas_image: None,
        }
    }

    /// This should be called whenever a texture is added to the ui_manager
    pub fn reload_font_atlas(&mut self, painter: &Painter) {
        if let Some(id) = self.font_atlas_image {
            // Remove possible font atlas texture.
            painter.release_image(id);
        }

        let mut fonts = self.fonts();

        // Create font texture and upload it.
        let handle = fonts.build_rgba32_texture();

        let id = painter.create_image(&ImageDescriptor {
            label: Some("Imgui Font Atlas"),
            width: handle.width,
            height: handle.height,
            srgb: false,
            renderable: false,
        });

        painter.write_image(id, handle.data);
        // Clear imgui texture data to save memory.
        fonts.clear_tex_data();
        fonts.tex_id = id;

        drop(fonts);

        self.font_atlas_image = Some(id);
    }

    /// Returns the dpi factor of the ui.
    pub fn hidpi_factor(&self) -> f64 {
        self.dpi.factor
    }

    /// Returns the imgui context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the imgui context
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn fonts(&mut self) -> FontAtlasRefMut<'_> {
        self.context.fonts()
    }

    pub fn io(&self) -> &Io {
        self.context.io()
    }

    pub fn io_mut(&mut self) -> &mut Io {
        self.context.io_mut()
    }

    /// Begins the creation of a new frame.
    pub fn frame<F: Fn(&mut Ui<'_>)>(&mut self, f: F) {
        if self.context.io().want_set_mouse_pos {
            let logical_pos = self.dpi.adjust_logical_pos(
                &self.window,
                LogicalPosition::new(
                    f64::from(self.context.io().mouse_pos[0]),
                    f64::from(self.context.io().mouse_pos[1]),
                ),
            );
            self.window.set_cursor_position(logical_pos).unwrap();
        }
        let mut ui = self.context.frame();
        f(&mut ui);

        let io = ui.io();
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            let cursor = CursorSettings {
                cursor: ui.mouse_cursor(),
                draw_cursor: io.mouse_draw_cursor,
            };
            if self.cursor_cache != Some(cursor) {
                cursor.apply(&self.window);
                self.cursor_cache = Some(cursor);
            }
        }

        ui.render();
    }

    pub fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        if self.window.id() == id {
            match event {
                WindowEvent::ModifiersChanged(mods) => {
                    let mut io = self.io_mut();
                    io.key_shift = mods.shift();
                    io.key_ctrl = mods.ctrl();
                    io.key_alt = mods.alt();
                    io.key_super = mods.logo();
                }
                WindowEvent::Resized(physical_size) => {
                    let logical_size = physical_size.to_logical::<f64>(self.window.scale_factor());
                    let logical_size = self.dpi.adjust_logical_size(&self.window, logical_size);
                    let mut io = self.io_mut();
                    io.display_size = [logical_size.width as f32, logical_size.height as f32];
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let hidpi_factor = *scale_factor;
                    // let mut io = self.io_mut();
                    // Mouse position needs to be changed while we still have both the old and the new
                    // values
                    if self.io_mut().mouse_pos[0].is_finite()
                        && self.io_mut().mouse_pos[1].is_finite()
                    {
                        self.io_mut().mouse_pos = [
                            self.io_mut().mouse_pos[0] * (hidpi_factor / self.dpi.factor) as f32,
                            self.io_mut().mouse_pos[1] * (hidpi_factor / self.dpi.factor) as f32,
                        ];
                    }
                    self.dpi.factor = hidpi_factor;
                    self.io_mut().display_framebuffer_scale =
                        [hidpi_factor as f32, hidpi_factor as f32];
                    // Window size might change too if we are using DPI rounding
                    let logical_size = self.window.inner_size().to_logical(*scale_factor);
                    let logical_size = self.dpi.adjust_logical_size(&self.window, logical_size);
                    self.io_mut().display_size =
                        [logical_size.width as f32, logical_size.height as f32];
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => {
                    let mut io = self.io_mut();
                    let pressed = *state == ElementState::Pressed;
                    io.keys_down[*key as usize] = pressed;

                    // This is a bit redundant here, but we'll leave it in. The OS occasionally
                    // fails to send modifiers keys, but it doesn't seem to send false-positives,
                    // so double checking isn't terrible in case some system *doesn't* send
                    // device events sometimes.
                    match key {
                        VirtualKeyCode::LShift | VirtualKeyCode::RShift => io.key_shift = pressed,
                        VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
                            io.key_ctrl = pressed
                        }
                        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => io.key_alt = pressed,
                        VirtualKeyCode::LWin | VirtualKeyCode::RWin => io.key_super = pressed,
                        _ => (),
                    }
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    // Exclude the backspace key ('\u{7f}'). Otherwise we will insert this char and then
                    // delete it.
                    if *ch != '\u{7f}' {
                        self.io_mut().add_input_character(*ch)
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = position.to_logical(self.window.scale_factor());
                    let position = self.dpi.adjust_logical_pos(&self.window, position);
                    self.io_mut().mouse_pos = [position.x as f32, position.y as f32];
                }
                WindowEvent::MouseWheel {
                    delta,
                    phase: TouchPhase::Moved,
                    ..
                } => match delta {
                    MouseScrollDelta::LineDelta(h, v) => {
                        let mut io = self.io_mut();
                        io.mouse_wheel_h = *h;
                        io.mouse_wheel = *v;
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        let pos = pos.to_logical::<f64>(self.dpi.factor);
                        let mut io = self.io_mut();
                        match pos.x.partial_cmp(&0.0) {
                            Some(Ordering::Greater) => io.mouse_wheel_h += 1.0,
                            Some(Ordering::Less) => io.mouse_wheel_h -= 1.0,
                            _ => (),
                        }
                        match pos.y.partial_cmp(&0.0) {
                            Some(Ordering::Greater) => io.mouse_wheel += 1.0,
                            Some(Ordering::Less) => io.mouse_wheel -= 1.0,
                            _ => (),
                        }
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    let io = self.io_mut();
                    let pressed = *state == ElementState::Pressed;
                    match button {
                        MouseButton::Left | MouseButton::Other(0) => {
                            io[imgui::MouseButton::Left] = pressed
                        }
                        MouseButton::Right | MouseButton::Other(1) => {
                            io[imgui::MouseButton::Right] = pressed
                        }
                        MouseButton::Middle | MouseButton::Other(2) => {
                            io[imgui::MouseButton::Middle] = pressed
                        }
                        MouseButton::Other(3) => io[imgui::MouseButton::Extra1] = pressed,
                        MouseButton::Other(4) => io[imgui::MouseButton::Extra2] = pressed,
                        _ => (),
                    }
                }
                _ => {}
            }
        }
    }

    pub fn on_device_event(&mut self, _id: DeviceId, event: &DeviceEvent) {
        let mut io = self.io_mut();
        match event {
            // Track key release events outside our window. If we don't do this,
            // we might never see the release event if some other window gets focus.
            DeviceEvent::Key(KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(key),
                ..
            }) => {
                io.keys_down[*key as usize] = false;
            }
            _ => {}
        }
    }
}
