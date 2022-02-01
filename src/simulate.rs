use clap::ArgMatches;
use libloading::Library;
use std::error::Error;
use std::path::PathBuf;

use constellation_base::project::{view::StarData, Project, View};
use constellation_base::Star;

use glam::DVec3;

use std::os::raw::c_double;
use std::os::raw::c_uint;

use crate::native;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let relative_path = PathBuf::from(
        matches
            .value_of("path")
            .ok_or("Must provide path variable")?,
    );
    let working_directory = std::env::current_dir()?;

    let project_directory = working_directory.join(relative_path);

    let mut project = Project::load(project_directory)?;

    invoke_solver(&mut project)?;

    project.save()?;

    Ok(())
}

fn invoke_solver(project: &mut Project) -> Result<(), Box<dyn Error>> {
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

    let context_destroy =
        unsafe { lib.get::<unsafe extern "C" fn(*mut native::Context)>(b"context_destroy")? };

    let n_body_source_create = unsafe {
        lib.get::<unsafe extern "C" fn() -> *mut native::NBodySource>(b"n_body_source_create")?
    };

    let n_body_source_add = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySource, native::NBody)>(
            b"n_body_source_add",
        )?
    };

    let n_body_source_destroy = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySource)>(b"n_body_source_destroy")?
    };

    let n_body_source_data_max_time = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySourceData) -> c_double>(
            b"n_body_source_data_max_time",
        )?
    };

    let n_body_source_data_steps = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySourceData) -> c_uint>(
            b"n_body_source_data_steps",
        )?
    };

    let n_body_source_data_n = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySourceData) -> c_uint>(
            b"n_body_source_data_n",
        )?
    };

    let n_body_source_data_slice = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::NBodySourceData, c_uint) -> *mut native::NBody>(
            b"n_body_source_data_slice",
        )?
    };

    let post_newtonian_solver_create = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::Context) -> *mut native::PostNewtonianSolver>(
            b"post_newtonian_solver_create",
        )?
    };

    let post_newtonian_solver_attach_n_body_source = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::PostNewtonianSolver, *mut native::NBodySource)>(
            b"post_newtonian_solver_attach_n_body_source",
        )?
    };

    let post_newtonian_solver_detach_n_body_source = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::PostNewtonianSolver, *mut native::NBodySource)>(
            b"post_newtonian_solver_detach_n_body_source",
        )?
    };

    let post_newtonian_solver_n_body_source_data = unsafe {
        lib.get::<unsafe extern "C" fn(
            *mut native::PostNewtonianSolver,
            *mut native::NBodySource,
        ) -> *mut native::NBodySourceData>(b"post_newtonian_solver_n_body_source_data")?
    };

    let post_newtonian_solver_run = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::PostNewtonianSolver, c_double, c_uint)>(
            b"post_newtonian_solver_run",
        )?
    };

    let post_newtonian_solver_destroy = unsafe {
        lib.get::<unsafe extern "C" fn(*mut native::PostNewtonianSolver)>(
            b"post_newtonian_solver_destroy",
        )?
    };

    let mut stars = Vec::<Star>::new();

    stars.push(Star {
        temp: 0.0,
        pos: DVec3::new(0.0, 0.0, 0.0),
        vel: DVec3::new(0.0, 0.0, 0.0),
        mass: 10e10,
    });

    stars.push(Star {
        temp: 0.0,
        pos: DVec3::new(10.0, 0.0, 0.0),
        vel: DVec3::new(0.0, 1.5, 0.0),
        mass: 1000.0,
    });

    unsafe {
        let descriptor = native::ContextDescriptor {
            speed_of_light: 299792458.0,
            gravitational_constant: 6.67408 * 10e-11,
        };

        let context = context_create(descriptor);

        let n_body_source = n_body_source_create();

        for star in stars.iter() {
            n_body_source_add(
                n_body_source,
                native::NBody {
                    x: star.pos.x,
                    y: star.pos.y,
                    z: star.pos.z,
                    velx: star.vel.x,
                    vely: star.vel.y,
                    velz: star.vel.z,
                    mass: star.mass,
                },
            );
        }

        let solver = post_newtonian_solver_create(context);

        post_newtonian_solver_attach_n_body_source(solver, n_body_source);

        post_newtonian_solver_run(solver, 0.01, 2000);

        let n_body_source_data = post_newtonian_solver_n_body_source_data(solver, n_body_source);

        let max_time = n_body_source_data_max_time(n_body_source_data);
        let steps = n_body_source_data_steps(n_body_source_data);
        let n = n_body_source_data_n(n_body_source_data);

        let view = View::new(max_time, steps as usize - 1, stars, |time, data| {
            let index = (time / max_time).round() as usize;

            let data_slice = std::slice::from_raw_parts(
                n_body_source_data_slice(n_body_source_data, index as u32),
                n as usize,
            );

            for i in 0..data_slice.len() {
                data[i] = StarData {
                    pos: DVec3::new(data_slice[i].x, data_slice[i].y, data_slice[i].z),
                    vel: DVec3::new(data_slice[i].velx, data_slice[i].vely, data_slice[i].velz),
                }
            }
        });

        project.insert_view("post_newtonian".into(), view);

        post_newtonian_solver_detach_n_body_source(solver, n_body_source);

        post_newtonian_solver_destroy(solver);

        n_body_source_destroy(n_body_source);

        context_destroy(context);
    }

    lib.close()?;

    Ok(())
}
