use super::State;

use egui::{CtxRef, Ui};

pub struct HomeState {}

impl HomeState {
    pub fn new() -> Self {
        Self {}
    }
}

impl State for HomeState {
    fn view(&mut self, ui: &mut Ui) {
        ui.strong("WELCOME TO CONSTELLATION ENGINE");
        ui.separator();
        ui.horizontal_wrapped( |ui| { 
            ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations. It is developed by Lukas Mesicek, a physics undergrad at the University of Utah."); 
        });
    }
    fn show(&mut self, _ctx: &CtxRef) {}
    fn update(&mut self) {}
    fn title(&mut self) -> &str {
        "Welcome" 
    }
}
