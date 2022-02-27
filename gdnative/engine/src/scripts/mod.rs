use gdnative::prelude::*;

mod grav_descriptor;
mod system_hierarchy;
mod system_manager;

use grav_descriptor::GravDescriptor;
use system_hierarchy::{SystemHierarchy, SystemHierarchyFormat, SystemHierarchyRoot};
use system_manager::SystemManager;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<SystemManager>();
    handle.add_class::<SystemHierarchy>();
    handle.add_class::<GravDescriptor>();
}

godot_init!(init);
