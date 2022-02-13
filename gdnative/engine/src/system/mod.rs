mod node;

pub mod object;
pub mod solver;

pub use node::{
    ObjectId, Objects, ObjectsMut, SystemError, SystemId, SystemNode, SystemNodeChildren,
    SystemNodes, SystemNodesMut,
};
pub use object::Object;
pub use solver::{EmptySolver, Solver};

use std::any::Any;

pub trait SystemRegister {
    fn register<S: System>(&mut self);
}

pub trait ObjectRegister {
    fn register<O: Object>(&mut self);
}

pub trait System: Any {
    type Solver: Solver;

    fn register_subsystems<R: SystemRegister>(&self, _reg: &mut R);
    fn register_objects<R: ObjectRegister>(&self, _reg: &mut R);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Root;

    impl System for Root {
        type Solver = EmptySolver;

        fn register_subsystems<R: SystemRegister>(&self, reg: &mut R) {
            reg.register::<SubSystem>();
        }

        fn register_objects<R: ObjectRegister>(&self, reg: &mut R) {}
    }

    struct SubSystem;

    impl System for SubSystem {
        type Solver = EmptySolver;

        fn register_subsystems<R: SystemRegister>(&self, _reg: &mut R) {}

        fn register_objects<R: ObjectRegister>(&self, reg: &mut R) {
            reg.register::<SubObject>();
        }
    }

    struct SubObject;

    impl Object for SubObject {}

    #[test]
    fn system_manager() {
        let mut root = SystemNode::new(Root, EmptySolver);
        let mut subsystem = SystemNode::new(SubSystem, EmptySolver);

        let object_id = subsystem.add_object(SubObject).unwrap();
        let subsystem_id = root.add_system_node(subsystem).unwrap();

        for (id, system) in root.system_nodes::<SubSystem>().unwrap() {
            assert!(id == subsystem_id);
            for (id, _object) in system.objects::<SubObject>().unwrap() {
                assert!(id == object_id);
            }
        }

        root.remove_system_node(subsystem_id).unwrap();

        assert!(root.system_nodes::<SubSystem>().unwrap().count() == 0);
    }
}
