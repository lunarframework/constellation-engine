use clap::{App, Arg};
use std::error::Error;

pub mod new;
pub mod simulate;

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
        .subcommand(App::new("simulate")
            .about("Simulate a project")
            .author("Lukas M.")
            .arg(Arg::new("path")
                .required(true)
                .index(1)))
        .get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => new::run(sub_matches)?,
        Some(("simulate", sub_matches)) => simulate::run(sub_matches)?,
        _ => {}
    };

    Ok(())
}
