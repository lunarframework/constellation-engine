use std::os::raw::c_double;
use std::os::raw::c_uint;

#[repr(C)]
pub struct PostNewtonianSolver;

#[repr(C)]
pub struct RingSolver;

#[repr(C)]
pub struct Domain {
    pub refinement: c_uint,
    pub outer_ratio: c_double,
}

#[repr(C)]
pub struct Accuracy {
    pub lapse_iterations: c_uint,
    pub lapse_residual: c_double,
    pub metric_iterations: c_uint,
    pub metric_residual: c_double,
    pub extrinsic_iterations: c_uint,
    pub extrinsic_residual: c_double,
}
