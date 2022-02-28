use std::os::raw::c_double;

#[repr(C)]
pub struct Constants {
    pub gravitational: c_double,
    pub speed_of_light: c_double,
}
