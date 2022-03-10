#![allow(dead_code)]

use crate::constants::Constants;
use std::ffi::c_void;
use std::os::raw::{c_double, c_int};

#[repr(C)]
pub struct Particle {
    pub x: c_double,
    pub y: c_double,
    pub z: c_double,
    pub velx: c_double,
    pub vely: c_double,
    pub velz: c_double,
    pub mass: c_double,
}

#[repr(C)]
pub struct ParticleSolverDescriptor {
    pub constants: Constants,

    pub particle_count: c_int,
    pub particles: *const Particle,

    pub element_order: c_int,

    pub domain_width: c_double,
    pub domain_height: c_double,
    pub domain_depth: c_double,
    pub domain_refinement: c_int,
}

#[repr(transparent)]
pub struct ParticleSolver(*mut c_void);

extern "C" {
    pub fn particle_solver_create(desc: ParticleSolverDescriptor) -> ParticleSolver;

    pub fn particle_solver_destroy(solver: ParticleSolver);
}
