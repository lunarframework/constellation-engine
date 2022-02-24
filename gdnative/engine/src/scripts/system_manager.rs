use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct SystemManager {}

#[methods]
impl SystemManager {
    /// The "constructor" of the class.
    fn new(_owner: &Reference) -> Self {
        Self {}
    }
}
