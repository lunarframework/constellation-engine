use imgui::{BackendFlags, ConfigFlags, Context, FontAtlasRefMut, Io, Key, Ui};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase,
    VirtualKeyCode, WindowEvent,
};
use winit::window::{CursorIcon, Window, WindowId};

use std::cmp::Ordering;
use std::sync::Arc;

use super::{
    PaintData, PaintElement, PaintIdx, PaintVtx, Painter, Palette, PaletteDescriptor, PaletteId,
};

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
    cursor_cache: Option<CursorSettings>,
    font_atlas: Option<Palette>,
    scale_factor: f64,
}

impl UiManager {
    /// Creates the gui handler and attaches it to the given window
    pub fn new(window: Arc<Window>) -> Self {
        let mut context = Context::create();

        context.set_platform_name(Some(format!(
            "constellation-engine {}",
            env!("CARGO_PKG_VERSION")
        )));

        context.set_renderer_name(Some(format!(
            "constellation-engine {}",
            env!("CARGO_PKG_VERSION")
        )));

        {
            let io = context.io_mut();
            io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
            io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
            io.backend_flags
                .insert(BackendFlags::RENDERER_HAS_VTX_OFFSET);
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

            io.display_framebuffer_scale =
                [window.scale_factor() as f32, window.scale_factor() as f32];
            let logical_size = window.inner_size().to_logical::<f32>(window.scale_factor());
            io.display_size = [logical_size.width as f32, logical_size.height as f32];
        }

        Self {
            context,
            scale_factor: window.scale_factor(),
            window,
            cursor_cache: Option::<CursorSettings>::None,
            font_atlas: None,
        }
    }

    /// This should be called whenever a texture is added to the ui_manager
    pub fn reload_font_atlas(&mut self, painter: &Painter) {
        let mut fonts = self.fonts();

        // Create font texture and upload it.
        let handle = fonts.build_rgba32_texture();

        let palette = painter.create_palette(&PaletteDescriptor {
            label: Some("Imgui Font Atlas"),
            width: handle.width,
            height: handle.height,
            srgb: false,
            renderable: false,
        });

        painter.write_palette(&palette, handle.data);
        // Clear imgui texture data to save memory.
        fonts.clear_tex_data();
        fonts.tex_id = palette.image_id();

        drop(fonts);

        self.font_atlas = Some(palette);
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
    pub fn frame<F: Fn(&mut Ui<'_>)>(&mut self, data: &mut PaintData, f: F) {
        if self.context.io().want_set_mouse_pos {
            self.window
                .set_cursor_position(LogicalPosition::new(
                    f64::from(self.context.io().mouse_pos[0]),
                    f64::from(self.context.io().mouse_pos[1]),
                ))
                .unwrap();
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

        let draw_data = ui.render();

        assert_eq!(
            std::mem::size_of::<imgui::DrawVert>(),
            std::mem::size_of::<PaintVtx>()
        );

        assert_eq!(
            std::mem::size_of::<imgui::DrawIdx>(),
            std::mem::size_of::<PaintIdx>()
        );

        let mut total_vtx_count = 0;
        let mut total_idx_count = 0;

        let mut elem_count = 0;

        for draw_list in draw_data.draw_lists() {
            elem_count += draw_list
                .commands()
                .filter(|command| {
                    if let imgui::DrawCmd::Elements { .. } = command {
                        return true;
                    };
                    false
                })
                .count();

            total_vtx_count += draw_list.vtx_buffer().len();
            total_idx_count += draw_list.idx_buffer().len();
        }
        data.set_pos(draw_data.display_pos);
        data.set_size(draw_data.display_size);

        data.reserve(total_vtx_count, total_idx_count, elem_count);

        let mut global_vtx_offset = 0;
        let mut global_idx_offset = 0;

        for draw_list in draw_data.draw_lists() {
            // Safety, imgui::DrawVert and PaintVtx should be the same size and layout
            // As should imgui::DrawIdx and PaintIdx.
            unsafe {
                use std::mem::transmute;
                data.add_vtx_sub_buffer(transmute(draw_list.vtx_buffer()));
                data.add_idx_sub_buffer(transmute(draw_list.idx_buffer()));
            }
            for draw_cmd in draw_list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = draw_cmd {
                    data.add_element(PaintElement {
                        idx_count: count,
                        clip_rect: cmd_params.clip_rect,
                        idx_offset: global_idx_offset + cmd_params.idx_offset,
                        vtx_offset: global_vtx_offset + cmd_params.vtx_offset,
                        palette_id: PaletteId::from(cmd_params.texture_id),
                    });
                };
            }

            global_vtx_offset += draw_list.vtx_buffer().len();
            global_idx_offset += draw_list.idx_buffer().len();
        }
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
                    let mut io = self.io_mut();
                    io.display_size = [logical_size.width as f32, logical_size.height as f32];
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let new_scale_factor = *scale_factor;
                    // let mut io = self.io_mut();
                    // Mouse position needs to be changed while we still have both the old and the new
                    // values
                    if self.io_mut().mouse_pos[0].is_finite()
                        && self.io_mut().mouse_pos[1].is_finite()
                    {
                        self.io_mut().mouse_pos = [
                            self.io_mut().mouse_pos[0]
                                * (new_scale_factor / self.scale_factor) as f32,
                            self.io_mut().mouse_pos[1]
                                * (new_scale_factor / self.scale_factor) as f32,
                        ];
                    }
                    self.scale_factor = new_scale_factor;
                    self.io_mut().display_framebuffer_scale =
                        [new_scale_factor as f32, new_scale_factor as f32];
                    // Window size might change too if we are using DPI rounding
                    let logical_size = self.window.inner_size().to_logical::<f32>(*scale_factor);
                    self.io_mut().display_size = [logical_size.width, logical_size.height];
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
                    let position = position.to_logical::<f32>(self.window.scale_factor());
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
                        let pos = pos.to_logical::<f64>(self.scale_factor);
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
