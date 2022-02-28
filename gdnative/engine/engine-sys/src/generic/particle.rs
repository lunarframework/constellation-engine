#![allow(dead_code)]

use std::ffi::c_void;

#[repr(transparent)]
pub struct ParticleSolver(c_void);

extern "C" {
    fn particle_solver_create() -> ParticleSolver;

    fn particle_solver_destroy(solver: ParticleSolver);
}
