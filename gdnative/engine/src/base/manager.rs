use super::Solver;
use super::{Object, ObjectRegister, System, SystemRegister};
use crate::heap::Iter as HeapIter;
use crate::heap::IterMut as HeapIterMut;
use crate::heap::{Heap, HeapError, HeapPointer};
use hashbrown::HashMap;
use std::cell::RefCell;
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemManagerError {
    #[error("The given identifier was invalid")]
    InvalidId,
    #[error("The given type has not been registered")]
    UnregisteredType,
}

impl From<HeapError> for SystemManagerError {
    fn from(error: HeapError) -> Self {
        match error {
            HeapError::InvalidPointer(..) => SystemManagerError::InvalidId,
            HeapError::UnregisteredType(..) => SystemManagerError::UnregisteredType,
        }
    }
}

/// An identifier to a system stored in a system manager
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SystemId(HeapPointer);

/// An identifier to an object that is a child of a system
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ObjectId(HeapPointer);

/// A System Manager or tree. It stores the hierarchy of systems and objects connected to a given root,
/// and manages deletion, iteration, and serialization
pub struct SystemManager {
    base: SystemContext,
}

impl SystemManager {
    /// Creates a new system manager from a given root.
    pub fn new<S: System>(root: S, solver: S::Solver) -> Self {
        let mut systems = Heap::new();
        systems.register::<SystemStorage<S>>();

        Self::register_subsystems(&mut systems, &root);
        let children = Self::create_child_heap(&root);

        let root = SystemId(
            systems
                .insert(SystemStorage {
                    system: root,
                    solver,
                })
                .unwrap(),
        );

        Self {
            systems,
            hierarchy: HashMap::new(),
            root,
            root_children: children,
        }
    }

    /// Returns the system id of the root system
    pub fn root(&self) -> SystemId {
        self.root
    }

    /// Invokes a system's solver on that system
    pub fn solve<S: System>(&mut self, system: SystemId) -> Result<(), SystemManagerError> {
        let subsystem = self.systems.get_mut::<SystemStorage<S>>(system.0)?;

        // subsystem.solver.solve(self);

        Ok(())
    }

    pub fn system<S: System>(&self, system: SystemId) -> Result<&S, SystemManagerError> {
        Ok(&self.systems.get::<SystemStorage<S>>(system.0)?.system)
    }

    pub fn system_mut<S: System>(
        &mut self,
        system: SystemId,
    ) -> Result<&mut S, SystemManagerError> {
        Ok(&mut self.systems.get_mut::<SystemStorage<S>>(system.0)?.system)
    }

    /// Adds a subsystem to a parent system
    pub fn add_subsystem<S: System>(
        &mut self,
        parent: SystemId,
        system: S,
        solver: S::Solver,
    ) -> Result<SystemId, SystemManagerError> {
        Self::register_subsystems(&mut self.systems, &system);

        let children = Self::create_child_heap(&system);

        // assert!(self.system_exists(parent));
        let system = SystemId(self.systems.insert(SystemStorage { system, solver })?);

        let pointer = self.children_mut(parent).insert(SystemPtr::<S> {
            system,
            _marker: PhantomData::<S>,
        })?;

        let context = SystemContext {
            _parent: parent,
            pointer,
            children: RefCell::new(children),
        };

        self.hierarchy.insert(system, context);

        Ok(system)
    }

    /// Removes a subsystem from a parent system
    pub fn remove_subsystem(
        &mut self,
        parent: SystemId,
        system: SystemId,
    ) -> Result<(), SystemManagerError> {
        // assert!(self.system_exists(system));

        let context = self.hierarchy.remove(&system).unwrap();

        self.children_mut(parent).remove(context.pointer)?;

        self.systems.remove(system.0)?;

        Ok(())
    }

    /// Adds an object to a system
    pub fn add_object<O: Object>(
        &mut self,
        parent: SystemId,
        object: O,
    ) -> Result<ObjectId, SystemManagerError> {
        Ok(ObjectId(
            self.children_mut(parent)
                .insert(ObjectStorage::<O> { object })?,
        ))
    }

    /// Removes an object from a system
    pub fn remove_object(
        &mut self,
        parent: SystemId,
        object: ObjectId,
    ) -> Result<(), SystemManagerError> {
        self.children_mut(parent).remove(object.0)?;

        Ok(())
    }

    // /// Iterates non-mutably through all subsystems of a given parent.
    // pub fn subsystems<S: System>(
    //     &self,
    //     parent: SystemId,
    // ) -> Result<SubSystems<'_, S>, SystemManagerError> {
    //     Ok(SubSystems::<S> {
    //         systems: &self.systems,
    //         children: self.children(parent).iter::<SubSystemPtr<S>>()?,
    //     })
    // }

    // /// Iterates mutably through all subsystems of a given parent.
    // pub fn subsystems_mut<S: System>(
    //     &mut self,
    //     parent: SystemId,
    // ) -> Result<SubSystemsMut<'_, S>, SystemManagerError> {
    //     // let iter_mut = self.children_mut(parent).iter_mut::<SubSystemPtr<S>>();

    //     let iter_mut = if parent != self.root {
    //         self.hierarchy
    //             .get_mut(&parent)
    //             .unwrap()
    //             .children
    //             .iter_mut::<SubSystemPtr<S>>()
    //     } else {
    //         self.root_children.iter_mut::<SubSystemPtr<S>>()
    //     }?;

    //     Ok(SubSystemsMut::<S> {
    //         systems: &mut self.systems,
    //         children: iter_mut,
    //     })
    // }

    // /// Iterates through the objects of a given system
    // pub fn objects<O: Object>(
    //     &self,
    //     parent: SystemId,
    // ) -> Result<Objects<'_, O>, SystemManagerError> {
    //     Ok(Objects::<O> {
    //         children: self.children(parent).iter::<Child<O>>()?,
    //     })
    // }

    // /// Iterates mutably through the objects of a system.
    // pub fn objects_mut<O: Object>(
    //     &mut self,
    //     parent: SystemId,
    // ) -> Result<ObjectsMut<'_, O>, SystemManagerError> {
    //     Ok(ObjectsMut::<O> {
    //         children: self.children_mut(parent).iter_mut::<Child<O>>()?,
    //     })
    // }

    fn children(&self, parent: SystemId) -> &Heap {
        if parent != self.root {
            &self.hierarchy.get(&parent).unwrap().children
        } else {
            &self.root_children
        }
    }

    fn children_mut(&mut self, parent: SystemId) -> &mut Heap {
        if parent != self.root {
            &mut self.hierarchy.get_mut(&parent).unwrap().children
        } else {
            &mut self.root_children
        }
    }

    fn register_subsystems(systems: &mut Heap, system: &impl System) {
        let mut register = SystemStorageRegister { heap: systems };

        system.register_subsystems(&mut register);
    }

    fn create_child_heap(system: &impl System) -> Heap {
        let mut heap = Heap::new();

        let mut object_register = ObjectStorageRegister { heap: &mut heap };

        system.register_objects(&mut object_register);

        let mut system_ptr_register = SystemPtrRegister { heap: &mut heap };

        system.register_subsystems(&mut system_ptr_register);

        heap
    }
}

// ***********************
// Context ***************
// ***********************

struct SystemContext {
    // Parent system
    _parent: SystemId,
    // Pointer to current system in parent heap
    pointer: HeapPointer,
    // Heap that stores children of current systems
    children: RefCell<Heap>,
}

// **********************
// Iterators ************
// **********************

pub struct SystemView<'a, S: System> {
    systems: &'a mut Heap,
    children: &'a mut Heap,
}

// pub struct SubSystems<'a, S: System> {
//     systems: &'a Heap,
//     children: HeapIter<'a, SubSystemPtr<S>>,
// }

// impl<'a, S: System> Iterator for SubSystems<'a, S> {
//     type Item = (SystemId, &'a S);

//     fn next(&mut self) -> Option<Self::Item> {
//         let subsystem = self.children.next()?.1.system;
//         let system = self.systems.get::<S>(subsystem.0).unwrap();

//         Some((subsystem, system))
//     }
// }

// pub struct SubSystemsMut<'a, S: System> {
//     systems: &'a mut Heap,
//     children: HeapIterMut<'a, SubSystemPtr<S>>,
// }

// impl<'a, S: System> Iterator for SubSystemsMut<'a, S> {
//     type Item = (SystemId, &'a mut S);

//     fn next(&mut self) -> Option<Self::Item> {
//         let subsystem = self.children.next()?.1.system;

//         unsafe {
//             let system_ptr = self.systems.get_mut::<S>(subsystem.0).unwrap() as *mut S;

//             Some((subsystem, system_ptr.as_mut().unwrap()))
//         }
//     }
// }

// pub struct Objects<'a, O: Object> {
//     children: HeapIter<'a, Child<O>>,
// }

// impl<'a, O: Object> Iterator for Objects<'a, O> {
//     type Item = (ObjectId, &'a O);

//     fn next(&mut self) -> Option<Self::Item> {
//         let child = self.children.next()?;

//         Some((ObjectId(child.0), &child.1.object))
//     }
// }

// pub struct ObjectsMut<'a, O: Object> {
//     children: HeapIterMut<'a, Child<O>>,
// }

// impl<'a, O: Object> Iterator for ObjectsMut<'a, O> {
//     type Item = (ObjectId, &'a mut O);

//     fn next(&mut self) -> Option<Self::Item> {
//         let child = self.children.next()?;

//         Some((ObjectId(child.0), &mut child.1.object))
//     }
// }

// Registration

struct ObjectStorage<O: Object> {
    object: O,
}

struct ObjectStorageRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> ObjectRegister for ObjectStorageRegister<'a> {
    fn register<O: Object>(&mut self) {
        self.heap.register::<ObjectStorage<O>>();
    }
}

struct SystemStorage<S: System> {
    solver: S::Solver,
    system: S,
}

struct SystemStorageRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> SystemRegister for SystemStorageRegister<'a> {
    fn register<S: System>(&mut self) {
        self.heap.register::<SystemStorage<S>>();
    }
}
