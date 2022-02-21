mod math;
mod node;
mod record;
mod tree;

use hecs::serialize::column::{DeserializeContext, SerializeContext};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;

pub use hecs::World;
pub use math::AbstractVector;
pub use node::SystemNode;
pub use record::ContinuousRecord;
pub use tree::{SystemConfig, SystemTree};

pub trait Object: Any {}

pub trait System: Send + Sync + Any {
    /// Sets up the system and adds any potential subsystems to the world.
    /// Recursively calls setup on any subsystems
    fn setup(
        &mut self,
        children: &mut World,
        config: &SystemConfig,
        start_time: f64,
        end_time: f64,
    );

    /// Update the system and all subsystems
    fn update(&mut self, children: &mut World, config: &SystemConfig, delta: f64);
}

pub trait SerializableSystem<'de>: Serialize + Deserialize<'de> {
    type SerContext: SerializeContext;
    type DeContext: DeserializeContext;

    fn ser_context() -> Self::SerContext;
    fn de_context() -> Self::DeContext;
}

pub trait Root {
    fn init_config(config: &mut SystemConfig);
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
