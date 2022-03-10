use gdnative::prelude::*;

#[derive(NativeClass, Clone)]
#[inherit(Reference)]
pub struct UnitsDescriptor {
    #[property]
    pub mass: i32,
    #[property]
    pub length: i32,
    #[property]
    pub time: i32,
}

#[methods]
impl UnitsDescriptor {
    fn new(_owner: &Reference) -> Self {
        Self::default()
    }
}

impl Default for UnitsDescriptor {
    fn default() -> Self {
        Self {
            mass: 0,
            length: 0,
            time: 0,
        }
    }
}
