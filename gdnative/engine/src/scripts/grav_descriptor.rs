use gdnative::{api::Gradient, prelude::*};

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
