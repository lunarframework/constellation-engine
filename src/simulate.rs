use libloading::{Library, Symbol};

#[repr(C)]
struct Solver3d;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Saftey: Called on valid platform
    let lib = unsafe { Library::new("solver.cdylib")? };

    // Saftey: valid symbol names and signatures
    let create_solver_3d =
        unsafe { lib.get::<unsafe extern "C" fn() -> *mut Solver3d>("create_solver_3d")? };
    let destroy_solver_3d =
        unsafe { lib.get::<unsafe extern "C" fn(*mut Solver3d)>("destroy_solver_3d")? };

    lib.close();

    Ok(())
}
