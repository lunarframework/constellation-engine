use crate::StateManager;

pub struct MainPanel {}

impl MainPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ctx: &egui::CtxRef, manager: &mut StateManager) {
        egui::CentralPanel::default().show(ctx, |ui| manager.view(ui));
    }
}
