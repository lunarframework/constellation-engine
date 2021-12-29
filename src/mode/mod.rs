pub mod open;
pub mod test;

use crate::{App, AppEvent, AppState};
use clap::ArgMatches;

pub fn new(_matches: &ArgMatches) {}

pub fn welcome(_matches: &ArgMatches) {
    App::new().run(|event| match event {
        AppEvent::CloseRequested => AppState::Exit,
        AppEvent::Frame { ctx, .. } => {
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.strong("WELCOME TO CONSTELLATION ENGINE");
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                // ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations"); 
                ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations. It is developed by Lukas Mesicek, a physics undergrad at the University of Utah."); 
                });
            });
            AppState::Run
        }
        _ => AppState::Run,
    });
}

pub use open::open;
pub use test::test;
