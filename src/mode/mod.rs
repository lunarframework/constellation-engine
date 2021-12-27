pub mod open;

use crate::{launch, ConsoleApp};
use clap::ArgMatches;

pub fn new(_matches: &ArgMatches) {}

struct WelcomeApp;

impl ConsoleApp for WelcomeApp {
    fn update(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.strong("WELCOME TO CONSTELLATION ENGINE");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                // ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations"); 
                ui.label("Constellation Engine is a tool for creating, editing, and visualizing physics simulations. It is developed by Lukas Mesicek, a physics undergrad at the University of Utah."); 
            });
        });
    }
}

pub fn welcome(_matches: &ArgMatches) {
    launch(WelcomeApp);
}

pub use open::open;
