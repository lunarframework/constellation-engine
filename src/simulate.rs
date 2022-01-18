use clap::ArgMatches;
use libloading::Library;
use std::error::Error;
use std::path::PathBuf;

use constellation_base::project::Project;

#[repr(C)]
struct GenericSolver;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let relative_path = PathBuf::from(
        matches
            .value_of("path")
            .ok_or("Must provide path variable")?,
    );
    let working_directory = std::env::current_dir()?;

    let project_directory = working_directory.join(relative_path);

    let project = Project::load(project_directory)?;

    // Saftey: Called on valid platform
    let lib = unsafe { Library::new("solver.cdylib")? };

    // Saftey: valid symbol names and signatures
    let create_generic_solver = unsafe {
        lib.get::<unsafe extern "C" fn() -> *mut GenericSolver>(b"create_generic_solver")?
    };

    let run_generic_solver =
        unsafe { lib.get::<unsafe extern "C" fn(*mut GenericSolver)>(b"run_generic_solver")? };

    let destroy_generic_solver =
        unsafe { lib.get::<unsafe extern "C" fn(*mut GenericSolver)>(b"destroy_generic_solver")? };

    unsafe {
        let generic_solver = create_generic_solver();

        run_generic_solver(generic_solver);

        destroy_generic_solver(generic_solver);
    }

    lib.close()?;

    project.save()?;

    Ok(())
}
