use clap::{App, Arg};
use std::error::Error;

pub mod new;

fn main() -> Result<(), Box<dyn Error>> {
    // std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // modes.push(Box::new(OpenMode::new()));

    let matches = clap::App::new("Constellation Engine")
        .bin_name("constellation")
        .version("0.1.0")
        .author("Lukas M.")
        .about("Relativistic physics simulation engine and viewer")
        .subcommand(App::new("new")
            .about("Creates new simulation file which can then be editted manually or via the edit command")
            .author("Lukas M.")
            .arg(Arg::new("path")
                .required(true)
                .index(1)))
        .subcommand(App::new("open")
            .about("Opens a simulation folder for edisting")
            .author("Lukas M.")
            .arg(Arg::new("path")
                .required(true)
                .index(1)))
        .subcommand(App::new("test")
            .about("Test")
            .author("Lukas M."))
        .subcommand(App::new("welcome")
            .about("Opens simple document detailing some of Constellation Engine's capabilities")
            .author("Lukas M."))
        .get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => new::run(sub_matches)?,
        _ => {}
    };

    Ok(())
}
