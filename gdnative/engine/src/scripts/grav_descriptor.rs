use gdnative::prelude::*;

#[derive(NativeClass, Clone)]
#[inherit(Reference)]
pub struct GravDescriptor {
    #[property]
    pub name: String,
}

#[methods]
impl GravDescriptor {
    fn new(_owner: &Reference) -> Self {
        Self::default()
    }
}

impl Default for GravDescriptor {
    fn default() -> Self {
        Self {
            name: String::from("Untitled"),
        }
    }
}

#[derive(NativeClass, Clone)]
#[inherit(Reference)]
pub struct NBodyStarDescriptor {
    #[property]
    pub pos: Vector3,
    #[property]
    pub vel: Vector3,
    #[property]
    pub mass: f64,
    #[property]
    pub temp: f64,
}

#[methods]
impl NBodyStarDescriptor {
    fn new(_owner: &Reference) -> Self {
        Self::default()
    }
}

impl Default for NBodyStarDescriptor {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, 0.0),
            vel: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            temp: 0.0,
        }
    }
}
