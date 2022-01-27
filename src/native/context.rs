use std::os::raw::c_double;

#[repr(C)]
pub struct Context;

#[repr(C)]
pub struct ContextDescriptor {
    pub speed_of_light: c_double,
    pub gravitational_constant: c_double,
}
