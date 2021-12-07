pub mod file;

use file::FileMenu;

use crate::StateManager;

use egui::CtxRef;

pub struct MenuBar {
    file: FileMenu,
    // misc: MiscUi,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            file: FileMenu::new(),
            // misc: MiscUi::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::CtxRef, manager: &mut StateManager) {
        egui::TopBottomPanel::top("Menu Bar Panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.add_enabled_ui(!self.file.is_dialog_open(), |ui| {
                    self.file.append(ui, manager);
                });
            });
        });

        self.file.show(ctx, manager);
    }
}

pub trait Dialog {
    fn new() -> Self;
    fn name(&self) -> &str;
    fn show(&mut self, ui: &mut egui::Ui, manager: &mut StateManager) -> bool;
}

struct Openable<D: Dialog>(Option<D>);

impl<D: Dialog> Openable<D> {
    fn new() -> Self {
        Self(None)
    }

    fn open(&mut self) {
        self.0 = Some(D::new());
    }

    fn is_open(&self) -> bool {
        self.0.is_some()
    }

    fn show(&mut self, ctx: &CtxRef, manager: &mut StateManager) {
        let mut window_opened = true;
        let mut dialog_close = false;
        if let Some(ref mut dialog) = self.0 {
            egui::Window::new(dialog.name())
                .open(&mut window_opened)
                .resizable(true)
                .default_width(300.0)
                .auto_sized()
                .show(ctx, |ui| {
                    dialog_close = dialog.show(ui, manager);
                });
        }

        if !window_opened || dialog_close {
            self.0 = None;
        }
    }
}
