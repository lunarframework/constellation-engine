pub mod file;
// pub mod misc;

pub use file::FileUi;
// pub use misc::MiscUi;

pub struct BaseUi {
    file: FileUi,
    // misc: MiscUi,
}

impl BaseUi {
    pub fn new() -> Self {
        Self {
            file: FileUi::new(),
            // misc: MiscUi::new(),
        }
    }

    pub fn ui(&mut self, ctx: egui::CtxRef) {
        egui::TopBottomPanel::top("Menu Bar Panel").show(&ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.file.append_menu(ui);
            });
        });

        self.file.show_active_windows(&ctx);
    }
}
