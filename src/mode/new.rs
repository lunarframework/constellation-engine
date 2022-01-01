use crate::project::Project;
use clap::ArgMatches;
use std::path::PathBuf;
use std::process::exit;

pub fn new(matches: &ArgMatches) {
    let relative_path = PathBuf::from(
        matches
            .value_of("path")
            .expect("Failed to parse relative path"),
    );
    let working_directory = std::env::current_dir().expect("Failed to find working directory");

    let project_directory = working_directory.join(relative_path);

    if let Err(error) = std::fs::create_dir(project_directory.as_path()) {
        eprintln!("Creating project directory failed");
        eprintln!("{}", error);

        match error.raw_os_error() {
            Some(code) => exit(code),
            None => exit(1),
        }
    }

    let project_directory = project_directory
        .canonicalize()
        .expect("Failed to normalize path");

    let name = String::from(
        project_directory
            .file_name()
            .expect("Failed to convert directory name to project name")
            .to_str()
            .expect("Failed to convert Os Str to normal string"),
    );

    Project::init(project_directory, name);
}
