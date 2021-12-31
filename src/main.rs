pub mod app;
pub mod components;
pub mod mode;
pub mod render;

fn main() {
    env_logger::init();

    let matches = clap::App::new("Constellation Engine")
        .bin_name("constellation")
        .version("0.1.0")
        .author("Lukas M.")
        .about("Relativistic physics simulation engine and viewer")
        .subcommand(clap::SubCommand::with_name("new")
            .about("Creates new simulation file which can then be editted manually or via the edit command")
            .author("Lukas M.")
            .arg(clap::Arg::with_name("path")
                .required(true)
                .index(1)))
        .subcommand(clap::SubCommand::with_name("open")
            .about("Opens a simulation folder for edisting")
            .author("Lukas M.")
            .arg(clap::Arg::with_name("path")
                .required(true)
                .index(1)))
        .subcommand(clap::SubCommand::with_name("test")
            .about("Test")
            .author("Lukas M."))
        .subcommand(clap::SubCommand::with_name("welcome")
            .about("Opens simple document detailing some of Constellation Engine's capabilities")
            .author("Lukas M."))
        .get_matches();

    match matches.subcommand() {
        ("new", Some(sub_matches)) => mode::new(sub_matches),
        ("welcome", Some(sub_matches)) => mode::welcome(sub_matches),
        ("open", Some(sub_matches)) => mode::open(sub_matches),
        ("test", Some(sub_matches)) => mode::test(sub_matches),
        _ => {}
    }
}
