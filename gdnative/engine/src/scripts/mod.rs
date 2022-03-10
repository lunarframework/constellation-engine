use gdnative::prelude::*;

pub mod grav_descriptor;
pub mod solve_descriptor;
pub mod system_manager;
pub mod system_tree;
pub mod units_descriptor;

use grav_descriptor::{GravDescriptor, NBodyStarDescriptor};
use solve_descriptor::SolveDescriptor;
use system_manager::SystemManager;
use system_tree::{SystemTreeGD, SystemTreeRoot};
use units_descriptor::UnitsDescriptor;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<SystemManager>();
    handle.add_class::<SystemTreeGD>();
    handle.add_class::<GravDescriptor>();
    handle.add_class::<NBodyStarDescriptor>();
    handle.add_class::<SolveDescriptor>();
    handle.add_class::<UnitsDescriptor>();
}

godot_init!(init);
