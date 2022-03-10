use gdnative::prelude::*;

#[derive(NativeClass, Clone)]
#[inherit(Reference)]
pub struct SolveDescriptor {
    #[property]
    pub start_time: f64,
    #[property]
    pub end_time: f64,
    #[property]
    pub iterations: i32,
}

#[methods]
impl SolveDescriptor {
    fn new(_owner: &Reference) -> Self {
        Self::default()
    }
}

impl Default for SolveDescriptor {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            end_time: 0.0,
            iterations: 0,
        }
    }
}
