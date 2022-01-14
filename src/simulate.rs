use clap::ArgMatches;
use libloading::Library;
use std::error::Error;

#[repr(C)]
struct Solver3d;

pub fn run(_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Saftey: Called on valid platform
    let lib = unsafe { Library::new("solver.cdylib")? };

    // Saftey: valid symbol names and signatures
    let _create_solver_3d =
        unsafe { lib.get::<unsafe extern "C" fn() -> *mut Solver3d>(b"create_solver_3d")? };
    let _destroy_solver_3d =
        unsafe { lib.get::<unsafe extern "C" fn(*mut Solver3d)>(b"destroy_solver_3d")? };

    lib.close()?;

    Ok(())
}
