use super::{Object, ObjectRegister, Solver, System, SystemRegister};
use crate::heap::Iter as HeapIter;
use crate::heap::IterMut as HeapIterMut;
use crate::heap::{Heap, HeapError, HeapPointer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("The given identifier was invalid")]
    InvalidId,
    #[error("The given type has not been registered")]
    UnregisteredType,
}

impl From<HeapError> for SystemError {
    fn from(error: HeapError) -> Self {
        match error {
            HeapError::InvalidPointer(..) => SystemError::InvalidId,
            HeapError::UnregisteredType(..) => SystemError::UnregisteredType,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SystemId(HeapPointer);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ObjectId(HeapPointer);

pub struct SystemNode<S: System> {
    system: S,
    solver: S::Solver,
    children: SystemNodeChildren,
}

impl<S: System> SystemNode<S> {
    pub fn new(system: S, solver: S::Solver) -> Self {
        let children = SystemNodeChildren::new(&system);

        Self {
            system,
            solver,
            children,
        }
    }

    pub fn get(&self) -> &S {
        &self.system
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.system
    }

    pub fn solver(&self) -> &S::Solver {
        &self.solver
    }

    pub fn solver_mut(&mut self) -> &mut S::Solver {
        &mut self.solver
    }

    pub fn solve(&mut self) {
        self.solver.solve(&mut self.system, &mut self.children)
    }

    pub fn children(&self) -> &SystemNodeChildren {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut SystemNodeChildren {
        &mut self.children
    }

    pub fn add_system_node<T: System>(
        &mut self,
        node: SystemNode<T>,
    ) -> Result<SystemId, SystemError> {
        Ok(self.children.add_system_node(node)?)
    }

    pub fn system_nodes<T: System>(&self) -> Result<SystemNodes<'_, T>, SystemError> {
        Ok(self.children.system_nodes::<T>()?)
    }

    pub fn system_nodes_mut<T: System>(&mut self) -> Result<SystemNodesMut<'_, T>, SystemError> {
        Ok(self.children.system_nodes_mut::<T>()?)
    }

    pub fn remove_system_node(&mut self, node: SystemId) -> Result<(), SystemError> {
        Ok(self.children.remove_system_node(node)?)
    }

    pub fn add_object<O: Object>(&mut self, object: O) -> Result<ObjectId, SystemError> {
        Ok(self.children.add_object(object)?)
    }

    pub fn remove_object(&mut self, object: ObjectId) -> Result<(), SystemError> {
        Ok(self.children.remove_object(object)?)
    }

    pub fn objects<O: Object>(&self) -> Result<Objects<'_, O>, SystemError> {
        Ok(self.children.objects::<O>()?)
    }

    pub fn objects_mut<O: Object>(&mut self) -> Result<ObjectsMut<'_, O>, SystemError> {
        Ok(self.children.objects_mut::<O>()?)
    }
}

pub struct SystemNodeChildren {
    children: Heap,
}

impl SystemNodeChildren {
    pub fn new<S: System>(system: &S) -> Self {
        let mut children = Heap::new();

        let mut object_register = ObjectStorageRegister {
            heap: &mut children,
        };

        system.register_objects(&mut object_register);

        let mut system_register = SystemStorageRegister {
            heap: &mut children,
        };

        system.register_subsystems(&mut system_register);

        Self { children }
    }

    pub fn add_system_node<T: System>(
        &mut self,
        node: SystemNode<T>,
    ) -> Result<SystemId, SystemError> {
        Ok(SystemId(self.children.insert(node)?))
    }

    pub fn remove_system_node(&mut self, node: SystemId) -> Result<(), SystemError> {
        Ok(self.children.remove(node.0)?)
    }

    pub fn system_nodes<T: System>(&self) -> Result<SystemNodes<'_, T>, SystemError> {
        Ok(SystemNodes {
            children: self.children.iter()?,
        })
    }

    pub fn system_nodes_mut<T: System>(&mut self) -> Result<SystemNodesMut<'_, T>, SystemError> {
        Ok(SystemNodesMut {
            children: self.children.iter_mut()?,
        })
    }

    pub fn add_object<O: Object>(&mut self, object: O) -> Result<ObjectId, SystemError> {
        Ok(ObjectId(self.children.insert(object)?))
    }

    pub fn remove_object(&mut self, object: ObjectId) -> Result<(), SystemError> {
        Ok(self.children.remove(object.0)?)
    }

    pub fn objects<O: Object>(&self) -> Result<Objects<'_, O>, SystemError> {
        Ok(Objects {
            children: self.children.iter()?,
        })
    }

    pub fn objects_mut<O: Object>(&mut self) -> Result<ObjectsMut<'_, O>, SystemError> {
        Ok(ObjectsMut {
            children: self.children.iter_mut()?,
        })
    }
}

pub struct SystemNodes<'a, S: System> {
    children: HeapIter<'a, SystemNode<S>>,
}

impl<'a, S: System> Iterator for SystemNodes<'a, S> {
    type Item = (SystemId, &'a SystemNode<S>);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((SystemId(child.0), child.1))
    }
}

pub struct SystemNodesMut<'a, S: System> {
    children: HeapIterMut<'a, SystemNode<S>>,
}

impl<'a, S: System> Iterator for SystemNodesMut<'a, S> {
    type Item = (SystemId, &'a mut SystemNode<S>);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((SystemId(child.0), child.1))
    }
}

pub struct Objects<'a, O: Object> {
    children: HeapIter<'a, O>,
}

impl<'a, O: Object> Iterator for Objects<'a, O> {
    type Item = (ObjectId, &'a O);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((ObjectId(child.0), child.1))
    }
}

pub struct ObjectsMut<'a, O: Object> {
    children: HeapIterMut<'a, O>,
}

impl<'a, O: Object> Iterator for ObjectsMut<'a, O> {
    type Item = (ObjectId, &'a mut O);

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.next()?;

        Some((ObjectId(child.0), child.1))
    }
}

// *******************
// Type Registers ****
// *******************
struct ObjectStorageRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> ObjectRegister for ObjectStorageRegister<'a> {
    fn register<O: Object>(&mut self) {
        self.heap.register::<O>();
    }
}

struct SystemStorageRegister<'a> {
    heap: &'a mut Heap,
}

impl<'a> SystemRegister for SystemStorageRegister<'a> {
    fn register<S: System>(&mut self) {
        self.heap.register::<SystemNode<S>>();
    }
}
