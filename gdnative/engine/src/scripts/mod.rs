use gdnative::prelude::*;

mod system_manager;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<system_manager::SystemManager>();
}

godot_init!(init);
