pub mod file;

use file::FileMenu;

use crate::StateManager;

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
                self.file.append(ui, manager);
            });
        });

        self.file.show(ctx, manager);
    }
}
