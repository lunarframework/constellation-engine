use std::os::raw::c_double;

#[derive(Debug)]
#[repr(C)]
pub struct NBody {
    pub x: c_double,
    pub y: c_double,
    pub z: c_double,
    pub velx: c_double,
    pub vely: c_double,
    pub velz: c_double,
    pub mass: c_double,
}

#[repr(C)]
pub struct NBodySource;

#[repr(C)]
pub struct NBodySourceData;
