use clap::ArgMatches;
use libloading::Library;
use std::error::Error;
use std::path::PathBuf;

use constellation_base::project::Project;

use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_double;
use std::os::raw::c_uint;

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

    let view = project_directory.join("views");

    let vacuum = view.join("vacuum");

    if !vacuum.exists() {
        std::fs::create_dir(vacuum.clone())?;
    }

    let project = Project::load(project_directory)?;

    // Saftey: Called on valid platform
    let lib = unsafe { Library::new("solver.cdylib")? };

    // Saftey: valid symbol names and signatures
    let generic_solver_create = unsafe {
        lib.get::<unsafe extern "C" fn(*const c_char) -> *mut GenericSolver>(
            b"generic_solver_create",
        )?
    };

    let generic_solver_set_time_domain = unsafe {
        lib.get::<unsafe extern "C" fn(*mut GenericSolver, c_uint, c_double)>(
            b"generic_solver_set_time_domain",
        )?
    };

    let generic_solver_run =
        unsafe { lib.get::<unsafe extern "C" fn(*mut GenericSolver)>(b"generic_solver_run")? };

    let generic_solver_destroy =
        unsafe { lib.get::<unsafe extern "C" fn(*mut GenericSolver)>(b"generic_solver_destroy")? };

    unsafe {
        let output_dir = CString::new(vacuum.as_os_str().to_str().unwrap().as_bytes()).unwrap();

        println!("{:?}", output_dir);

        let generic_solver = generic_solver_create(output_dir.as_ptr());

        generic_solver_set_time_domain(generic_solver, 2, 1.0);

        println!("Running");

        generic_solver_run(generic_solver);

        println!("Done");

        generic_solver_destroy(generic_solver);
    }

    lib.close()?;

    project.save()?;

    Ok(())
}
