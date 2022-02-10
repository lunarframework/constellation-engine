mod manager;

pub use manager::{SystemId, SystemManager};

use std::any::Any;

pub trait SystemRegister {
    fn register<S: System>(&mut self);
}

pub trait ObjectRegister {
    fn register<O: Object>(&mut self);
}

pub trait System: Any {
    fn register_subsystems<R: SystemRegister>(&mut self, _reg: &mut R) {}
    fn register_objects<R: ObjectRegister>(&mut self, _reg: &mut R) {}
}

pub trait Object: Any {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Root;

    impl System for Root {
        fn register_subsystems<R: SystemRegister>(&mut self, reg: &mut R) {
            reg.register::<SubSystem>();
        }

        fn register_objects<R: ObjectRegister>(&mut self, reg: &mut R) {
            reg.register::<SubObject>();
        }
    }

    struct SubSystem;

    impl System for SubSystem {}

    struct SubObject;

    impl Object for SubObject {}

    #[test]
    fn system_manager() {
        let mut system_manager = SystemManager::new(Root);

        let root = system_manager.root();

        let subsystem_id = system_manager.add_subsystem(root, SubSystem).unwrap();

        let object_id = system_manager.add_object(root, SubObject).unwrap();

        for subsystem in system_manager.subsystems::<SubSystem>(root).unwrap() {
            assert!(subsystem_id == subsystem.0);
        }

        for object in system_manager.objects::<SubObject>(root).unwrap() {
            assert!(object_id == object.0);
        }

        system_manager.remove_subsystem(root, subsystem_id).unwrap();

        system_manager.remove_object(root, object_id).unwrap();

        assert!(
            system_manager
                .subsystems::<SubSystem>(root)
                .unwrap()
                .count()
                == 0
        );
        assert!(system_manager.objects::<SubObject>(root).unwrap().count() == 0);
    }
}
