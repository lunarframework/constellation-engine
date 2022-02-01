use clap::ArgMatches;
use libloading::Library;
use std::error::Error;

use std::os::raw::c_double;
use std::os::raw::c_uint;

use crate::native;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let delta = matches.value_of("delta").unwrap().parse::<f64>()?;
    let steps = matches.value_of("steps").unwrap().parse::<u32>()?;
    let iterations = matches.value_of("iterations").unwrap().parse::<u32>()?;
    let residual = matches.value_of("residual").unwrap().parse::<f64>()?;

    // Saftey: Called on valid platform
    let lib = unsafe { Library::new("solver.cdylib")? };

    // Saftey: valid symbol names and signatures
    // let generic_solver_create = unsafe {
    //     lib.get::<unsafe extern "C" fn(*const c_char) -> *mut GenericSolver>(
    //         b"generic_solver_create",
    //     )?
    // };

    let context_create = unsafe {
        lib.get::<unsafe extern "C" fn(native::ContextDescriptor) -> *mut native::Context>(
            b"context_create",
        )?
    };

    let ring_solver_create = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::Context) -> *mut native::RingSolver>(
            b"ring_solver_create",
        )?
    };

    let ring_solver_run = unsafe {
        lib.get::<unsafe extern "C" fn(
            *mut native::RingSolver,
            c_double,
            c_uint,
            native::Domain,
            native::Accuracy,
            c_double,
        ) -> c_double>(b"ring_solver_run")?
    };

    let ring_solver_destroy = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::RingSolver) -> *mut native::RingSolver>(
            b"ring_solver_destroy",
        )?
    };

    let context_destroy =
        unsafe { lib.get::<unsafe extern "C" fn(*mut native::Context)>(b"context_destroy")? };

    unsafe {
        let descriptor = native::ContextDescriptor {
            speed_of_light: 1.0,
            gravitational_constant: 1.0,
        };

        let context = context_create(descriptor);

        let solver = ring_solver_create(context);

        let domain = native::Domain {
            refinement: 2,
            outer_ratio: 2.0,
        };

        let accuracy = native::Accuracy {
            lapse_iterations: iterations,
            lapse_residual: residual,
            metric_iterations: iterations,
            metric_residual: residual,
            extrinsic_iterations: iterations,
            extrinsic_residual: residual,
        };

        let result = ring_solver_run(solver, delta, steps, domain, accuracy, 1.0);

        println!("The computed standard deviation is {}", result);

        ring_solver_destroy(solver);

        context_destroy(context);
    }

    lib.close()?;

    Ok(())
}
