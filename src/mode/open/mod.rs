use crate::{launch, ConsoleApp};
use clap::ArgMatches;

struct OpenApp;

impl ConsoleApp for OpenApp {
    fn update(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.strong("WELCOME TO CONSTELLATION ENGINE");
        });
    }
}

pub fn open(_matches: &ArgMatches) {
    launch(OpenApp);
}
