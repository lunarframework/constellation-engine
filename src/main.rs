use clap::{App, Arg};
use std::error::Error;

pub mod native;
pub mod new;
pub mod ring;
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
        .subcommand(App::new("ring")
            .about("Simulate a ring around a blackhole")
            .author("Lukas M.")
            .arg(Arg::new("delta").takes_value(true).long("delta").required(true))
            .arg(Arg::new("steps").takes_value(true).long("steps").required(true))
            .arg(Arg::new("iterations").takes_value(true).long("iterations").required(true))
            .arg(Arg::new("residual").takes_value(true).long("residual").required(true))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => new::run(sub_matches)?,
        Some(("simulate", sub_matches)) => simulate::run(sub_matches)?,
        Some(("ring", sub_matches)) => ring::run(sub_matches)?,
        _ => {}
    };

    Ok(())
}
