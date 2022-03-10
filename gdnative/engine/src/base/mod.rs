mod math;
mod node;
mod record;
mod tree;

use serde::{Deserializer, Serializer};
use std::any::Any;

pub use hecs::{Entity, World};
pub use math::AbstractVector;
pub use node::SystemNode;
pub use record::ContinuousRecord;
pub use tree::{Config, SystemConfig, SystemTree};

pub trait Object: Send + Sync + Any {}

pub trait System: Send + Sync + Sized + Any {
    fn solve_begin(&mut self, children: &mut World, config: &SystemConfig, time: f64);

    fn solve_update(&mut self, children: &mut World, config: &SystemConfig, time: f64, delta: f64);

    fn solve_end(&mut self, children: &mut World, config: &SystemConfig, time: f64);

    fn view_begin(&mut self, children: &mut World, config: &SystemConfig, time: f64);

    fn view_set_time(&mut self, children: &mut World, config: &SystemConfig, time: f64);

    fn view_end(&mut self, children: &mut World, config: &SystemConfig, time: f64);

    fn serialize_system<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize_system<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn serialize_children<S>(children: &World, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize_children<'de, D>(deserializer: D) -> Result<World, D::Error>
    where
        D: Deserializer<'de>;
}

pub trait Root {
    fn default_config() -> SystemConfig;

    fn serialize_config<S>(config: &SystemConfig, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize_config<'de, D>(deserializer: D) -> Result<SystemConfig, D::Error>
    where
        D: Deserializer<'de>;
}

#[cfg(test)]
mod tests {
    // use super::*;

    // struct Root;

    // impl System for Root {
    //     fn setup(&mut self, _children: &mut World, _start_time: f64, _end_time: f64) {}
    //     fn update(&mut self, _children: &mut World, _delta: f64) {}
    // }

    // struct SubSystem;

    // impl System for SubSystem {
    //     fn setup(&mut self, _children: &mut World, _start_time: f64, _end_time: f64) {}
    //     fn update(&mut self, _children: &mut World, _delta: f64) {}
    // }

    // struct SubObject;

    // impl Object for SubObject {}

    // #[test]
    // fn system_manager() {
    //     // let mut root = SystemNode::new(Root);
    //     // let mut subsystem = SystemNode::new(SubSystem);

    //     // let object_id = subsystem.add_object(SubObject).unwrap();
    //     // let subsystem_id = root.add_system_node(subsystem).unwrap();

    //     // for (id, system) in root.system_nodes::<SubSystem>().unwrap() {
    //     //     assert!(id == subsystem_id);
    //     //     for (id, _object) in system.objects::<SubObject>().unwrap() {
    //     //         assert!(id == object_id);
    //     //     }
    //     // }

    //     // root.remove_system_node(subsystem_id).unwrap();

    //     // assert!(root.system_nodes::<SubSystem>().unwrap().count() == 0);
    // }
}
