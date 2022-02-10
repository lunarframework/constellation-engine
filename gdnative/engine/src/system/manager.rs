use super::{Object, ObjectRegister, System, SystemRegister};
use crate::heap::Iter as HeapIter;
use crate::heap::IterMut as HeapIterMut;
use crate::heap::{Heap, HeapError, HeapPointer};
use hashbrown::HashMap;
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
    systems: Heap,
    hierarchy: HashMap<SystemId, SystemContext>,

    root: SystemId,
    root_children: Heap,
}

impl SystemManager {
    /// Creates a new system manager from a given root.
    pub fn new<S: System>(mut root: S) -> Self {
        let mut systems = Heap::new();
        systems.register::<S>();

        Self::register_subsystems(&mut systems, &mut root);
        let children = Self::create_child_heap(&mut root);

        let root = SystemId(systems.insert(root).unwrap());

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

    /// Adds a subsystem to a parent system
    pub fn add_subsystem<S: System>(
        &mut self,
        parent: SystemId,
        mut system: S,
    ) -> Result<SystemId, SystemManagerError> {
        Self::register_subsystems(&mut self.systems, &mut system);

        let children = Self::create_child_heap(&mut system);

        // assert!(self.system_exists(parent));
        let system = SystemId(self.systems.insert(system)?);

        let pointer = self.children_mut(parent).insert(SubSystemPtr::<S> {
            system,
            _marker: PhantomData::<S>,
        })?;

        let context = SystemContext {
            _parent: parent,
            pointer,
            children,
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
            self.children_mut(parent).insert(Child::<O> { object })?,
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

    /// Iterates non-mutably through all subsystems of a given parent.
    pub fn subsystems<S: System>(
        &self,
        parent: SystemId,
    ) -> Result<SubSystems<'_, S>, SystemManagerError> {
        Ok(SubSystems::<S> {
            systems: &self.systems,
            children: self.children(parent).iter::<SubSystemPtr<S>>()?,
        })
    }

    /// Iterates mutably through all subsystems of a given parent.
    pub fn subsystems_mut<S: System>(
        &mut self,
        parent: SystemId,
    ) -> Result<SubSystemsMut<'_, S>, SystemManagerError> {
        // let iter_mut = self.children_mut(parent).iter_mut::<SubSystemPtr<S>>();

        let iter_mut = if parent != self.root {
            self.hierarchy
                .get_mut(&parent)
                .unwrap()
                .children
                .iter_mut::<SubSystemPtr<S>>()
        } else {
            self.root_children.iter_mut::<SubSystemPtr<S>>()
        }?;

        Ok(SubSystemsMut::<S> {
            systems: &mut self.systems,
            children: iter_mut,
        })
    }

    /// Iterates through the objects of a given system
    pub fn objects<O: Object>(
        &self,
        parent: SystemId,
    ) -> Result<Objects<'_, O>, SystemManagerError> {
        Ok(Objects::<O> {
            children: self.children(parent).iter::<Child<O>>()?,
        })
    }

    /// Iterates mutably through the objects of a system.
    pub fn objects_mut<O: Object>(
        &mut self,
        parent: SystemId,
    ) -> Result<ObjectsMut<'_, O>, SystemManagerError> {
        Ok(ObjectsMut::<O> {
            children: self.children_mut(parent).iter_mut::<Child<O>>()?,
        })
    }

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

    fn register_subsystems(systems: &mut Heap, system: &mut impl System) {
        let mut register = SubSystemRegister { heap: systems };

        system.register_subsystems(&mut register);
    }

    fn create_child_heap(system: &mut impl System) -> Heap {
        let mut heap = Heap::new();

        let mut child_register = ChildRegister { heap: &mut heap };

        system.register_objects(&mut child_register);

        let mut subsystem_ptr_register = SubSystemPtrRegister { heap: &mut heap };

        system.register_subsystems(&mut subsystem_ptr_register);

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
    children: Heap,
}

// **********************
// Iterators ************
// **********************

pub struct SubSystems<'a, S: System> {
    systems: &'a Heap,
    children: HeapIter<'a, SubSystemPtr<S>>,
}

impl<'a, S: System> Iterator for SubSystems<'a, S> {
    type Item = (SystemId, &'a S);

    fn next(&mut self) -> Option<Self::Item> {
        let subsystem = self.children.next()?.1.system;
        let system = self.systems.get::<S>(subsystem.0).unwrap();

        Some((subsystem, system))
    }
}

pub struct SubSystemsMut<'a, S: System> {
    systems: &'a mut Heap,
    children: HeapIterMut<'a, SubSystemPtr<S>>,
}

impl<'a, S: System> Iterator for SubSystemsMut<'a, S> {
    type Item = (SystemId, &'a mut S);

    fn next(&mut self) -> Option<Self::Item> {
        let subsystem = self.children.next()?.1.system;

        unsafe {
            let system_ptr = self.systems.get_mut::<S>(subsystem.0).unwrap() as *mut S;

            Some((subsystem, system_ptr.as_mut().unwrap()))
        }
    }
}

pub struct Objects<'a, O: Object> {
    children: HeapIter<'a, Child<O>>,
}

impl<'a, O: Object> Iterator for Objects<'a, O> {
    type Item = (ObjectId, &'a O);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((ObjectId(child.0), &child.1.object))
    }
}

pub struct ObjectsMut<'a, O: Object> {
    children: HeapIterMut<'a, Child<O>>,
}

impl<'a, O: Object> Iterator for ObjectsMut<'a, O> {
    type Item = (ObjectId, &'a mut O);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((ObjectId(child.0), &mut child.1.object))
    }
}

// Registration

struct Child<O: Object> {
    object: O,
}

struct ChildRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> ObjectRegister for ChildRegister<'a> {
    fn register<O: Object>(&mut self) {
        self.heap.register::<Child<O>>();
    }
}

struct SubSystemRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> SystemRegister for SubSystemRegister<'a> {
    fn register<S: System>(&mut self) {
        self.heap.register::<S>();
    }
}

struct SubSystemPtr<S: System> {
    system: SystemId,
    _marker: PhantomData<S>,
}

struct SubSystemPtrRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> SystemRegister for SubSystemPtrRegister<'a> {
    fn register<S: System>(&mut self) {
        self.heap.register::<SubSystemPtr<S>>();
    }
}
