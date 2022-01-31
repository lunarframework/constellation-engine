use gdnative::prelude::*;

mod project_manager;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Register the new `HelloWorld` type we just declared.
    handle.add_class::<project_manager::ProjectManager>();
}

godot_init!(init);
