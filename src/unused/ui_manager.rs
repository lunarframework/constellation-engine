// use imgui::{BackendFlags, ConfigFlags, Context, FontAtlasRefMut, Io, Key, StyleColor, Ui};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
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

/// Builder of ui and manages the various user interations with that ui.
/// This is a thin wrapper atop `imgui`, an immediete mode gui lib.
pub struct UiManager {
    window: Arc<Window>,
    scale_factor: f64,
    context: egui::CtxRef,
    raw_input: egui::RawInput,
    modifier_state: winit::event::ModifiersState,
    pointer_pos: Option<egui::Pos2>,
}

impl UiManager {
    /// Creates the gui handler and attaches it to the given window
    pub fn new(window: Arc<Window>) -> Self {
        let mut context = egui::CtxRef::default();
        context.set_fonts(egui::FontDefinitions::default());
        context.set_style(egui::Style::default());

        let physical_width = window.inner_size().width;
        let physical_height = window.inner_size().height;
        let scale_factor = window.scale_factor();

        let raw_input = egui::RawInput {
            pixels_per_point: Some(scale_factor as f32),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::default(),
                egui::vec2(physical_width as f32, physical_height as f32) / scale_factor as f32,
            )),
            ..Default::default()
        };

        Self {
            window,
            scale_factor,
            context,
            raw_input,
            modifier_state: winit::event::ModifiersState::empty(),
            pointer_pos: Some(egui::Pos2::default()),
        }

        // context.set_platform_name(Some(format!(
        //     "constellation-engine {}",
        //     env!("CARGO_PKG_VERSION")
        // )));

        // context.set_renderer_name(Some(format!(
        //     "constellation-engine {}",
        //     env!("CARGO_PKG_VERSION")
        // )));

        // {
        //     let io = context.io_mut();
        //     io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        //     io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        //     io.backend_flags
        //         .insert(BackendFlags::RENDERER_HAS_VTX_OFFSET);
        //     io[Key::Tab] = VirtualKeyCode::Tab as _;
        //     io[Key::LeftArrow] = VirtualKeyCode::Left as _;
        //     io[Key::RightArrow] = VirtualKeyCode::Right as _;
        //     io[Key::UpArrow] = VirtualKeyCode::Up as _;
        //     io[Key::DownArrow] = VirtualKeyCode::Down as _;
        //     io[Key::PageUp] = VirtualKeyCode::PageUp as _;
        //     io[Key::PageDown] = VirtualKeyCode::PageDown as _;
        //     io[Key::Home] = VirtualKeyCode::Home as _;
        //     io[Key::End] = VirtualKeyCode::End as _;
        //     io[Key::Insert] = VirtualKeyCode::Insert as _;
        //     io[Key::Delete] = VirtualKeyCode::Delete as _;
        //     io[Key::Backspace] = VirtualKeyCode::Back as _;
        //     io[Key::Space] = VirtualKeyCode::Space as _;
        //     io[Key::Enter] = VirtualKeyCode::Return as _;
        //     io[Key::Escape] = VirtualKeyCode::Escape as _;
        //     io[Key::KeyPadEnter] = VirtualKeyCode::NumpadEnter as _;
        //     io[Key::A] = VirtualKeyCode::A as _;
        //     io[Key::C] = VirtualKeyCode::C as _;
        //     io[Key::V] = VirtualKeyCode::V as _;
        //     io[Key::X] = VirtualKeyCode::X as _;
        //     io[Key::Y] = VirtualKeyCode::Y as _;
        //     io[Key::Z] = VirtualKeyCode::Z as _;

        //     io.display_framebuffer_scale =
        //         [window.scale_factor() as f32, window.scale_factor() as f32];
        //     let logical_size = window.inner_size().to_logical::<f32>(window.scale_factor());
        //     io.display_size = [logical_size.width as f32, logical_size.height as f32];
        // }
        // {
        //     // style[StyleColor::WindowBg] = [0.1, 0.105, 0.11, 1.0];

        //     // // Headers
        //     // style[StyleColor::Header] = [0.2, 0.205, 0.21, 1.0];
        //     // style[StyleColor::HeaderHovered] = [0.3, 0.305, 0.31, 1.0];
        //     // style[StyleColor::HeaderActive] = [0.15, 0.1505, 0.151, 1.0];

        //     // // Buttons
        //     // style[StyleColor::Button] = [0.2, 0.205, 0.21, 1.0];
        //     // style[StyleColor::ButtonHovered] = [0.3, 0.305, 0.31, 1.0];
        //     // style[StyleColor::ButtonActive] = [0.15, 0.1505, 0.151, 1.0];

        //     // // Frame background
        //     // style[StyleColor::FrameBg] = [0.2, 0.205, 0.21, 1.0];
        //     // style[StyleColor::FrameBgHovered] = [0.3, 0.305, 0.31, 1.0];
        //     // style[StyleColor::FrameBgActive] = [0.15, 0.1505, 0.151, 1.0];

        //     // // Tabs
        //     // style[StyleColor::Tab] = [0.15, 0.1505, 0.151, 1.0];
        //     // style[StyleColor::TabHovered] = [0.38, 0.3805, 0.381, 1.0];
        //     // style[StyleColor::TabActive] = [0.28, 0.2805, 0.281, 1.0];
        //     // style[StyleColor::TabUnfocused] = [0.15, 0.1505, 0.151, 1.0];
        //     // style[StyleColor::TabUnfocusedActive] = [0.2, 0.205, 0.21, 1.0];

        //     // // Title
        //     // style[StyleColor::TitleBg] = [0.15, 0.1505, 0.151, 1.0];
        //     // style[StyleColor::TitleBgActive] = [0.15, 0.1505, 0.151, 1.0];
        //     // style[StyleColor::TitleBgCollapsed] = [0.15, 0.1505, 0.151, 1.0];

        //     // // Resize grip
        //     // style[StyleColor::ResizeGrip] = [0.91, 0.91, 0.91, 0.25];
        //     // style[StyleColor::ResizeGripHovered] = [0.81, 0.81, 0.81, 0.67];
        //     // style[StyleColor::ResizeGripActive] = [0.46, 0.46, 0.46, 0.95];

        //     // // Scrollbar
        //     // style[StyleColor::ScrollbarBg] = [0.02, 0.02, 0.02, 0.53];
        //     // style[StyleColor::ScrollbarGrab] = [0.31, 0.31, 0.31, 1.0];
        //     // style[StyleColor::ScrollbarGrabHovered] = [0.41, 0.41, 0.41, 1.0];
        //     // style[StyleColor::ScrollbarGrabActive] = [0.51, 0.51, 0.51, 1.0];

        //     // // Check mark
        //     // style[StyleColor::CheckMark] = [0.94, 0.94, 0.94, 1.0];

        //     // // Slider
        //     // style[StyleColor::SliderGrab] = [0.51, 0.51, 0.51, 0.7];
        //     // style[StyleColor::CheckMark] = [0.66, 0.66, 0.66, 1.0];

        //     fn color_from_rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
        //         [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
        //     }

        //     let bg_color = color_from_rgb(7, 7, 8);
        //     let light_bg_color = color_from_rgb(32, 32, 35);
        //     let very_light_bg_color = color_from_rgb(50, 50, 55);

        //     let title_color = color_from_rgb(23, 1, 71);
        //     let title_color_active = color_from_rgb(30, 1, 107);
        //     let title_color_collapsed = color_from_rgb(18, 1, 54);

        //     let panel_color = color_from_rgb(21, 21, 25);
        //     let panel_hover_color = color_from_rgb(29, 151, 236);
        //     let panel_active_color = color_from_rgb(0, 119, 200);

        //     let text_color = color_from_rgb(255, 255, 255);
        //     let text_disabled_color = color_from_rgb(151, 151, 151);
        //     let border_color = color_from_rgb(48, 48, 48);

        //     let style = &mut context.style_mut();

        //     style[StyleColor::Text] = text_color;
        //     style[StyleColor::TextDisabled] = text_disabled_color;
        //     style[StyleColor::TextSelectedBg] = panel_active_color;
        //     style[StyleColor::WindowBg] = bg_color;
        //     style[StyleColor::ChildBg] = bg_color;
        //     style[StyleColor::PopupBg] = bg_color;
        //     style[StyleColor::Border] = border_color;
        //     style[StyleColor::BorderShadow] = border_color;
        //     style[StyleColor::FrameBg] = panel_color;
        //     style[StyleColor::FrameBgHovered] = panel_hover_color;
        //     style[StyleColor::FrameBgActive] = panel_active_color;
        //     style[StyleColor::TitleBg] = title_color;
        //     style[StyleColor::TitleBgActive] = title_color_active;
        //     style[StyleColor::TitleBgCollapsed] = title_color_collapsed;
        //     style[StyleColor::MenuBarBg] = panel_color;
        //     style[StyleColor::ScrollbarBg] = panel_color;
        //     style[StyleColor::ScrollbarGrab] = light_bg_color;
        //     style[StyleColor::ScrollbarGrabHovered] = very_light_bg_color;
        //     style[StyleColor::ScrollbarGrabActive] = very_light_bg_color;
        //     style[StyleColor::CheckMark] = panel_active_color;
        //     style[StyleColor::SliderGrab] = panel_hover_color;
        //     style[StyleColor::SliderGrabActive] = panel_active_color;
        //     style[StyleColor::Button] = panel_color;
        //     style[StyleColor::ButtonHovered] = panel_hover_color;
        //     style[StyleColor::ButtonActive] = panel_hover_color;
        //     style[StyleColor::Header] = panel_color;
        //     style[StyleColor::HeaderHovered] = panel_hover_color;
        //     style[StyleColor::HeaderActive] = panel_active_color;
        //     style[StyleColor::Separator] = border_color;
        //     style[StyleColor::SeparatorHovered] = border_color;
        //     style[StyleColor::SeparatorActive] = border_color;
        //     style[StyleColor::ResizeGrip] = bg_color;
        //     style[StyleColor::ResizeGripHovered] = panel_color;
        //     style[StyleColor::ResizeGripActive] = light_bg_color;
        //     style[StyleColor::PlotLines] = panel_active_color;
        //     style[StyleColor::PlotLinesHovered] = panel_hover_color;
        //     style[StyleColor::PlotHistogram] = panel_active_color;
        //     style[StyleColor::PlotHistogramHovered] = panel_hover_color;
        //     style[StyleColor::DragDropTarget] = bg_color;
        //     style[StyleColor::NavHighlight] = bg_color;
        //     style[StyleColor::Tab] = bg_color;
        //     style[StyleColor::TabActive] = panel_active_color;
        //     style[StyleColor::TabUnfocused] = bg_color;
        //     style[StyleColor::TabUnfocusedActive] = panel_active_color;
        //     style[StyleColor::TabHovered] = panel_hover_color;

        //     style.window_rounding = 0.0;
        //     style.child_rounding = 0.0;
        //     style.frame_rounding = 0.0;
        //     style.grab_rounding = 0.0;
        //     style.popup_rounding = 0.0;
        //     style.scrollbar_rounding = 0.0;
        //     style.tab_rounding = 0.0;
        // }

        // Self {
        //     context,
        //     scale_factor: window.scale_factor(),
        //     window,
        //     cursor_cache: Option::<CursorSettings>::None,
        //     font_atlas: None,
        // }
    }

    // /// This should be called whenever a texture is added to the ui_manager
    // pub fn reload_font_atlas(&mut self, painter: &Painter) {
    //     let mut fonts = self.fonts();

    //     // Create font texture and upload it.
    //     let handle = fonts.build_rgba32_texture();

    //     let palette = painter.create_palette(&PaletteDescriptor {
    //         label: Some("Imgui Font Atlas"),
    //         width: handle.width,
    //         height: handle.height,
    //         srgb: true,
    //         renderable: false,
    //     });

    //     painter.write_palette(&palette, handle.data);
    //     // Clear imgui texture data to save memory.
    //     fonts.clear_tex_data();
    //     fonts.tex_id = palette.image_id();

    //     drop(fonts);

    //     self.font_atlas = Some(palette);
    // }

    /// Returns the imgui context
    pub fn context(&self) -> egui::CtxRef {
        self.context.clone()
    }

    // /// Begins the creation of a new frame.
    // pub fn frame<F: FnOnce(&mut Ui<'_>)>(&mut self, data: &mut PaintData, f: F) {
    //     if self.context.io().want_set_mouse_pos {
    //         self.window
    //             .set_cursor_position(LogicalPosition::new(
    //                 f64::from(self.context.io().mouse_pos[0]),
    //                 f64::from(self.context.io().mouse_pos[1]),
    //             ))
    //             .unwrap();
    //     }
    //     let mut ui = self.context.frame();
    //     f(&mut ui);

    //     let io = ui.io();
    //     if !io
    //         .config_flags
    //         .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
    //     {
    //         let cursor = CursorSettings {
    //             cursor: ui.mouse_cursor(),
    //             draw_cursor: io.mouse_draw_cursor,
    //         };
    //         if self.cursor_cache != Some(cursor) {
    //             cursor.apply(&self.window);
    //             self.cursor_cache = Some(cursor);
    //         }
    //     }

    //     let draw_data = ui.render();

    //     assert_eq!(
    //         std::mem::size_of::<imgui::DrawVert>(),
    //         std::mem::size_of::<PaintVtx>()
    //     );

    //     assert_eq!(
    //         std::mem::size_of::<imgui::DrawIdx>(),
    //         std::mem::size_of::<PaintIdx>()
    //     );

    //     let mut total_vtx_count = 0;
    //     let mut total_idx_count = 0;

    //     let mut elem_count = 0;

    //     for draw_list in draw_data.draw_lists() {
    //         elem_count += draw_list
    //             .commands()
    //             .filter(|command| {
    //                 if let imgui::DrawCmd::Elements { .. } = command {
    //                     return true;
    //                 };
    //                 false
    //             })
    //             .count();

    //         total_vtx_count += draw_list.vtx_buffer().len();
    //         total_idx_count += draw_list.idx_buffer().len();
    //     }
    //     data.set_pos(draw_data.display_pos);
    //     data.set_size(draw_data.display_size);

    //     data.reserve(total_vtx_count, total_idx_count, elem_count);

    //     let mut global_vtx_offset = 0;
    //     let mut global_idx_offset = 0;

    //     for draw_list in draw_data.draw_lists() {
    //         // Safety, imgui::DrawVert and PaintVtx should be the same size and layout
    //         // As should imgui::DrawIdx and PaintIdx.
    //         unsafe {
    //             use std::mem::transmute;
    //             data.add_vtx_sub_buffer(transmute(draw_list.vtx_buffer()));
    //             data.add_idx_sub_buffer(transmute(draw_list.idx_buffer()));
    //         }
    //         for draw_cmd in draw_list.commands() {
    //             if let imgui::DrawCmd::Elements { count, cmd_params } = draw_cmd {
    //                 data.add_element(PaintElement {
    //                     idx_count: count,
    //                     clip_rect: cmd_params.clip_rect,
    //                     idx_offset: global_idx_offset + cmd_params.idx_offset,
    //                     vtx_offset: global_vtx_offset + cmd_params.vtx_offset,
    //                     palette_id: PaletteId::from(cmd_params.texture_id),
    //                 });
    //             };
    //         }

    //         global_vtx_offset += draw_list.vtx_buffer().len();
    //         global_idx_offset += draw_list.idx_buffer().len();
    //     }
    // }

    pub fn on_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        if self.window.id() == id {
            match event {
                // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                // See: https://github.com/rust-windowing/winit/issues/208
                // There is nothing to do for minimize events, so it is ignored here. This solves an issue where
                // egui window positions would be changed when minimizing on Windows.
                WindowEvent::Resized(PhysicalSize {
                    width: 0,
                    height: 0,
                }) => {}
                WindowEvent::Resized(physical_size) => {
                    self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                        Default::default(),
                        egui::vec2(physical_size.width as f32, physical_size.height as f32)
                            / self.scale_factor as f32,
                    ));
                }
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    self.scale_factor = *scale_factor;
                    self.raw_input.pixels_per_point = Some(*scale_factor as f32);
                    self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                        Default::default(),
                        egui::vec2(new_inner_size.width as f32, new_inner_size.height as f32)
                            / self.scale_factor as f32,
                    ));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let winit::event::MouseButton::Other(..) = button {
                    } else {
                        // push event only if the cursor is inside the window
                        if let Some(pointer_pos) = self.pointer_pos {
                            self.raw_input.events.push(egui::Event::PointerButton {
                                pos: pointer_pos,
                                button: match button {
                                    winit::event::MouseButton::Left => egui::PointerButton::Primary,
                                    winit::event::MouseButton::Right => {
                                        egui::PointerButton::Secondary
                                    }
                                    winit::event::MouseButton::Middle => {
                                        egui::PointerButton::Middle
                                    }
                                    winit::event::MouseButton::Other(_) => unreachable!(),
                                },
                                pressed: *state == winit::event::ElementState::Pressed,
                                modifiers: Default::default(),
                            });
                        }
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let mut delta = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            let line_height = 8.0; // TODO as in egui_glium
                            egui::vec2(*x, *y) * line_height
                        }
                        winit::event::MouseScrollDelta::PixelDelta(delta) => {
                            egui::vec2(delta.x as f32, delta.y as f32)
                        }
                    };

                    // The ctrl (cmd on macos) key indicates a zoom is desired.
                    if self.raw_input.modifiers.ctrl || self.raw_input.modifiers.command {
                        self.raw_input.zoom_delta *= (delta.y / 200.0).exp();
                    } else {
                        self.raw_input.scroll_delta += delta;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let pointer_pos = egui::pos2(
                        position.x as f32 / self.scale_factor as f32,
                        position.y as f32 / self.scale_factor as f32,
                    );
                    self.pointer_pos = Some(pointer_pos);
                    self.raw_input
                        .events
                        .push(egui::Event::PointerMoved(pointer_pos));
                }
                WindowEvent::CursorLeft { .. } => {
                    self.pointer_pos = None;
                    self.raw_input.events.push(egui::Event::PointerGone);
                }
                WindowEvent::ModifiersChanged(input) => {
                    self.modifier_state = *input;
                    self.raw_input.modifiers = winit_to_egui_modifiers(*input);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        let pressed = input.state == winit::event::ElementState::Pressed;

                        if pressed {
                            let is_ctrl = self.modifier_state.ctrl();
                            if is_ctrl && virtual_keycode == VirtualKeyCode::C {
                                self.raw_input.events.push(egui::Event::Copy)
                            } else if is_ctrl && virtual_keycode == VirtualKeyCode::X {
                                self.raw_input.events.push(egui::Event::Cut)
                            } else if is_ctrl && virtual_keycode == VirtualKeyCode::V {
                                // #[cfg(feature = "clipboard")]
                                // if let Some(ref mut clipboard) = self.clipboard {
                                //     if let Ok(contents) = clipboard.get_contents() {
                                //         self.raw_input.events.push(egui::Event::Text(contents))
                                //     }
                                // }
                            } else if let Some(key) = winit_to_egui_key_code(virtual_keycode) {
                                self.raw_input.events.push(egui::Event::Key {
                                    key,
                                    pressed: input.state == winit::event::ElementState::Pressed,
                                    modifiers: winit_to_egui_modifiers(self.modifier_state),
                                });
                            }
                        }
                    }
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    if is_printable(*ch)
                        && !self.modifier_state.ctrl()
                        && !self.modifier_state.logo()
                    {
                        self.raw_input
                            .events
                            .push(egui::Event::Text(ch.to_string()));
                    }
                }
                _ => {}
            }

            // match event {
            //     WindowEvent::ModifiersChanged(mods) => {
            //         let mut io = self.io_mut();
            //         io.key_shift = mods.shift();
            //         io.key_ctrl = mods.ctrl();
            //         io.key_alt = mods.alt();
            //         io.key_super = mods.logo();
            //     }
            //     WindowEvent::Resized(physical_size) => {
            //         let logical_size = physical_size.to_logical::<f64>(self.window.scale_factor());
            //         let mut io = self.io_mut();
            //         io.display_size = [logical_size.width as f32, logical_size.height as f32];
            //     }
            //     WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            //         let new_scale_factor = *scale_factor;
            //         // let mut io = self.io_mut();
            //         // Mouse position needs to be changed while we still have both the old and the new
            //         // values
            //         if self.io_mut().mouse_pos[0].is_finite()
            //             && self.io_mut().mouse_pos[1].is_finite()
            //         {
            //             self.io_mut().mouse_pos = [
            //                 self.io_mut().mouse_pos[0]
            //                     * (new_scale_factor / self.scale_factor) as f32,
            //                 self.io_mut().mouse_pos[1]
            //                     * (new_scale_factor / self.scale_factor) as f32,
            //             ];
            //         }
            //         self.scale_factor = new_scale_factor;
            //         self.io_mut().display_framebuffer_scale =
            //             [new_scale_factor as f32, new_scale_factor as f32];
            //         // Window size might change too if we are using DPI rounding
            //         let logical_size = self.window.inner_size().to_logical::<f32>(*scale_factor);
            //         self.io_mut().display_size = [logical_size.width, logical_size.height];
            //     }
            //     WindowEvent::KeyboardInput {
            //         input:
            //             KeyboardInput {
            //                 virtual_keycode: Some(key),
            //                 state,
            //                 ..
            //             },
            //         ..
            //     } => {
            //         let mut io = self.io_mut();
            //         let pressed = *state == ElementState::Pressed;
            //         io.keys_down[*key as usize] = pressed;

            //         // This is a bit redundant here, but we'll leave it in. The OS occasionally
            //         // fails to send modifiers keys, but it doesn't seem to send false-positives,
            //         // so double checking isn't terrible in case some system *doesn't* send
            //         // device events sometimes.
            //         match key {
            //             VirtualKeyCode::LShift | VirtualKeyCode::RShift => io.key_shift = pressed,
            //             VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
            //                 io.key_ctrl = pressed
            //             }
            //             VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => io.key_alt = pressed,
            //             VirtualKeyCode::LWin | VirtualKeyCode::RWin => io.key_super = pressed,
            //             _ => (),
            //         }
            //     }
            //     WindowEvent::ReceivedCharacter(ch) => {
            //         // Exclude the backspace key ('\u{7f}'). Otherwise we will insert this char and then
            //         // delete it.
            //         if *ch != '\u{7f}' {
            //             self.io_mut().add_input_character(*ch)
            //         }
            //     }
            //     WindowEvent::CursorMoved { position, .. } => {
            //         let position = position.to_logical::<f32>(self.window.scale_factor());
            //         self.io_mut().mouse_pos = [position.x as f32, position.y as f32];
            //     }
            //     WindowEvent::MouseWheel {
            //         delta,
            //         phase: TouchPhase::Moved,
            //         ..
            //     } => match delta {
            //         MouseScrollDelta::LineDelta(h, v) => {
            //             let mut io = self.io_mut();
            //             io.mouse_wheel_h = *h;
            //             io.mouse_wheel = *v;
            //         }
            //         MouseScrollDelta::PixelDelta(pos) => {
            //             let pos = pos.to_logical::<f64>(self.scale_factor);
            //             let mut io = self.io_mut();
            //             match pos.x.partial_cmp(&0.0) {
            //                 Some(Ordering::Greater) => io.mouse_wheel_h += 1.0,
            //                 Some(Ordering::Less) => io.mouse_wheel_h -= 1.0,
            //                 _ => (),
            //             }
            //             match pos.y.partial_cmp(&0.0) {
            //                 Some(Ordering::Greater) => io.mouse_wheel += 1.0,
            //                 Some(Ordering::Less) => io.mouse_wheel -= 1.0,
            //                 _ => (),
            //             }
            //         }
            //     },
            //     WindowEvent::MouseInput { state, button, .. } => {
            //         let io = self.io_mut();
            //         let pressed = *state == ElementState::Pressed;
            //         match button {
            //             MouseButton::Left | MouseButton::Other(0) => {
            //                 io[imgui::MouseButton::Left] = pressed
            //             }
            //             MouseButton::Right | MouseButton::Other(1) => {
            //                 io[imgui::MouseButton::Right] = pressed
            //             }
            //             MouseButton::Middle | MouseButton::Other(2) => {
            //                 io[imgui::MouseButton::Middle] = pressed
            //             }
            //             MouseButton::Other(3) => io[imgui::MouseButton::Extra1] = pressed,
            //             MouseButton::Other(4) => io[imgui::MouseButton::Extra2] = pressed,
            //             _ => (),
            //         }
            //     }
            //     _ => {}
            // }
        }
    }

    /// Updates the internal time for egui used for animations. `elapsed_seconds` should be the seconds since some point in time (for example application start).
    pub fn update_time(&mut self, elapsed_seconds: f64) {
        self.raw_input.time = Some(elapsed_seconds);
    }

    /// Starts a new frame by providing a new `Ui` instance to write into.
    pub fn begin_frame(&mut self) {
        self.context.begin_frame(self.raw_input.take());
    }

    /// Ends the frame. Returns what has happened as `Output` and gives you the draw instructions
    /// as `PaintJobs`. If the optional `window` is set, it will set the cursor key based on
    /// egui's instructions.
    pub fn end_frame(
        &mut self,
        window: Option<&winit::window::Window>,
    ) -> (egui::Output, Vec<egui::paint::ClippedShape>) {
        // otherwise the below line gets flagged by clippy if both clipboard and webbrowser features are disabled
        #[allow(clippy::let_and_return)]
        let parts = self.context.end_frame();

        if let Some(window) = window {
            if let Some(cursor_icon) = egui_to_winit_cursor_icon(parts.0.cursor_icon) {
                window.set_cursor_visible(true);
                // if the pointer is located inside the window, set cursor icon
                if self.pointer_pos.is_some() {
                    window.set_cursor_icon(cursor_icon);
                }
            } else {
                window.set_cursor_visible(false);
            }
        }

        // #[cfg(feature = "clipboard")]
        // handle_clipboard(&parts.0, self.clipboard.as_mut());

        parts
    }

    pub fn on_device_event(&mut self, _id: DeviceId, event: &DeviceEvent) {
        match event {
            _ => {}
        }
    }
}

/// Translates winit to egui keycodes.
#[inline]
fn winit_to_egui_key_code(key: VirtualKeyCode) -> Option<egui::Key> {
    Some(match key {
        Escape => egui::Key::Escape,
        Insert => egui::Key::Insert,
        Home => egui::Key::Home,
        Delete => egui::Key::Delete,
        End => egui::Key::End,
        PageDown => egui::Key::PageDown,
        PageUp => egui::Key::PageUp,
        Left => egui::Key::ArrowLeft,
        Up => egui::Key::ArrowUp,
        Right => egui::Key::ArrowRight,
        Down => egui::Key::ArrowDown,
        Back => egui::Key::Backspace,
        Return => egui::Key::Enter,
        Tab => egui::Key::Tab,
        Space => egui::Key::Space,

        A => egui::Key::A,
        K => egui::Key::K,
        U => egui::Key::U,
        W => egui::Key::W,
        Z => egui::Key::Z,

        _ => {
            return None;
        }
    })
}

/// Translates winit to egui modifier keys.
#[inline]
fn winit_to_egui_modifiers(modifiers: winit::event::ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        #[cfg(target_os = "macos")]
        mac_cmd: modifiers.logo(),
        #[cfg(target_os = "macos")]
        command: modifiers.logo(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: modifiers.ctrl(),
    }
}

#[inline]
fn egui_to_winit_cursor_icon(icon: egui::CursorIcon) -> Option<winit::window::CursorIcon> {
    use egui::CursorIcon::*;

    match icon {
        Default => Some(CursorIcon::Default),
        ContextMenu => Some(CursorIcon::ContextMenu),
        Help => Some(CursorIcon::Help),
        PointingHand => Some(CursorIcon::Hand),
        Progress => Some(CursorIcon::Progress),
        Wait => Some(CursorIcon::Wait),
        Cell => Some(CursorIcon::Cell),
        Crosshair => Some(CursorIcon::Crosshair),
        Text => Some(CursorIcon::Text),
        VerticalText => Some(CursorIcon::VerticalText),
        Alias => Some(CursorIcon::Alias),
        Copy => Some(CursorIcon::Copy),
        Move => Some(CursorIcon::Move),
        NoDrop => Some(CursorIcon::NoDrop),
        NotAllowed => Some(CursorIcon::NotAllowed),
        Grab => Some(CursorIcon::Grab),
        Grabbing => Some(CursorIcon::Grabbing),
        AllScroll => Some(CursorIcon::AllScroll),
        ResizeHorizontal => Some(CursorIcon::EwResize),
        ResizeNeSw => Some(CursorIcon::NeswResize),
        ResizeNwSe => Some(CursorIcon::NwseResize),
        ResizeVertical => Some(CursorIcon::NsResize),
        ZoomIn => Some(CursorIcon::ZoomIn),
        ZoomOut => Some(CursorIcon::ZoomOut),
        None => Option::None,
    }
}

/// We only want printable characters and ignore all special keys.
#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}
