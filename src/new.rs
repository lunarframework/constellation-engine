use clap::ArgMatches;
use std::error::Error;
use std::path::PathBuf;

use constellation_base::project::Project;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let relative_path = PathBuf::from(
        matches
            .value_of("path")
            .ok_or("Must provide path variable")?,
    );
    let working_directory = std::env::current_dir()?;

    let project_directory = working_directory.join(relative_path);

    std::fs::create_dir(project_directory.as_path())?;

    let project_directory = project_directory.canonicalize()?;

    let name = String::from(
        project_directory
            .file_name()
            .ok_or("Failed to find proj directory name")?
            .to_str()
            .ok_or("Failed to convert project directory to name")?,
    );

    Project::init(project_directory, name);

    Ok(())
}
