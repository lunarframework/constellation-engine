use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use hecs::{Entity, World};
use std::any::Any;
use std::any::TypeId;
use std::hash::Hash;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Invalid system id")]
    InvalidSystemId,
    #[error("Root System")]
    RootSystem,
}

pub trait System: Send + Sync + Any {
    fn on_create(&mut self, manager: SystemManager, time: f64);
    fn on_update(&mut self, manager: SystemManager, time: f64, delta: f64);
    fn on_destroy(&mut self, manager: SystemManager, time: f64);

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Root: Send + Sync + Any {}

pub trait Config: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Context<R: Root + System>: Send + Sync + Any {
    fn on_attach(&mut self, root: &R);
    fn on_detach(&mut self);

    fn on_edit_begin(&self, manager: SystemManager, system: SystemId);
    fn on_edit_end(&self, maanger: SystemManager, system: SystemId);

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SystemId(usize);

pub struct SystemManager {
    systems: HashMap<SystemId, SystemNode>,
    config: TypeIdMap<Box<dyn Config>>,

    free_list: Vec<SystemId>,
    next_id: SystemId,
}

impl SystemManager {
    pub fn attach_config<C: Config>(&mut self, config: C) {
        self.config.insert(TypeId::of::<C>(), Box::new(config));
    }

    pub fn detach_config<C: Config>(&mut self) {
        self.config.remove(&TypeId::of::<C>());
    }

    pub fn config<C: Config>(&self) -> &C {
        let config = self.config.get(&TypeId::of::<C>()).unwrap();
        config.as_any().downcast_ref().unwrap()
    }

    pub fn config_mut<C: Config>(&mut self) -> &mut C {
        let config = self.config.get_mut(&TypeId::of::<C>()).unwrap();
        config.as_any_mut().downcast_mut().unwrap()
    }

    pub fn entity(&self, id: SystemId) -> Result<Entity, SystemError> {
        Ok(self
            .systems
            .get(&id)
            .ok_or(SystemError::InvalidSystemId)?
            .entity
            .ok_or(SystemError::RootSystem)?)
    }

    pub fn children(&self, id: SystemId) -> Result<&World, SystemError> {
        Ok(&self
            .systems
            .get(&id)
            .ok_or(SystemError::InvalidSystemId)?
            .children)
    }

    pub fn children_mut(&self, id: SystemId) -> Result<&mut World, SystemError> {
        Ok(&mut self
            .systems
            .get_mut(&id)
            .ok_or(SystemError::InvalidSystemId)?
            .children)
    }

    pub fn add_system<S: System>(
        &mut self,
        parent: SystemId,
        system: S,
    ) -> Result<SystemId, SystemError> {
        let parent_node = self
            .systems
            .get_mut(&parent)
            .ok_or(SystemError::InvalidSystemId)?;

        let entity = parent_node.children.spawn((system,));

        let node = SystemNode {
            parent,
            entity: Some(entity),
            children: World::new(),
        };

        let id = self.next_id();

        self.systems.insert(id, node);

        Ok(id)
    }

    fn next_id(&mut self) -> SystemId {
        if let Some(last) = self.free_list.pop() {
            return last;
        }

        let id = self.next_id;
        self.next_id.0 += 1;
        id
    }
}

pub struct SystemTree<R: Root + System> {
    root: R,
    root_id: SystemId,
    manager: SystemManager,
    context: TypeIdMap<Box<dyn Context<R>>>,
}

impl<R: Root + System> SystemTree<R> {
    pub fn new(root: R) -> Self {
        let root_id = SystemId(0);

        let mut systems = HashMap::new();

        systems.insert(
            root_id,
            SystemNode {
                parent: root_id,
                entity: None,
                children: World::new(),
            },
        );

        Self {
            root,
            root_id,
            manager: SystemManager {
                systems,
                config: TypeIdMap::default(),
                free_list: Vec::new(),
                next_id: SystemId(1),
            },
            context: TypeIdMap::default(),
        }
    }

    pub fn root(&self) -> &R {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut R {
        &mut self.root
    }

    pub fn root_id(&self) -> SystemId {
        self.root_id
    }

    pub fn attach_config<C: Config>(&mut self, config: C) {
        self.manager
            .config
            .insert(TypeId::of::<C>(), Box::new(config));
    }

    pub fn detach_config<C: Config>(&mut self) {
        self.manager.config.remove(&TypeId::of::<C>());
    }

    pub fn config<C: Config>(&self) -> &C {
        self.manager.config()
    }

    pub fn config_mut<C: Config>(&mut self) -> &mut C {
        self.manager.config_mut()
    }

    pub fn attach_context<C: Context<R>>(&mut self, context: C) {
        context.on_attach(&self.root);
        self.context.insert(TypeId::of::<C>(), Box::new(context));
    }

    pub fn detach_context<C: Context<R>>(&mut self) {
        if let Some(context) = self.context.remove(&TypeId::of::<C>()) {
            context.on_detach();
        }
    }

    pub fn context<C: Context<R>>(&self) -> &C {
        let context = self.context.get(&TypeId::of::<C>()).unwrap();
        context.as_any().downcast_ref().unwrap()
    }

    pub fn context_mut<C: Context<R>>(&mut self) -> &mut C {
        let context = self.context.get_mut(&TypeId::of::<C>()).unwrap();
        context.as_any_mut().downcast_mut().unwrap()
    }

    pub fn entity(&self, id: SystemId) -> Result<Entity, SystemError> {
        self.manager.entity(id)
    }

    pub fn add_system<S: System>(
        &mut self,
        parent: SystemId,
        system: S,
    ) -> Result<SystemId, SystemError> {
        self.manager.add_system(parent, system)
    }
}

impl<R: System + Root> Drop for SystemTree<R> {
    fn drop(&mut self) {
        for context in self.context.iter_mut() {
            context.1.on_detach();
        }
    }
}

/// A hasher optimized for hashing a single TypeId.
///
/// TypeId is already thoroughly hashed, so there's no reason to hash it again.
/// Just leave the bits unchanged.
#[derive(Default)]
struct TypeIdHasher {
    hash: u64,
}

impl Hasher for TypeIdHasher {
    fn write_u64(&mut self, n: u64) {
        // Only a single value can be hashed, so the old hash should be zero.
        debug_assert_eq!(self.hash, 0);
        self.hash = n;
    }

    // Tolerate TypeId being either u64 or u128.
    fn write_u128(&mut self, n: u128) {
        debug_assert_eq!(self.hash, 0);
        self.hash = n as u64;
    }

    fn write(&mut self, bytes: &[u8]) {
        debug_assert_eq!(self.hash, 0);

        // This will only be called if TypeId is neither u64 nor u128, which is not anticipated.
        // In that case we'll just fall back to using a different hash implementation.
        let mut hasher = <DefaultHashBuilder as BuildHasher>::Hasher::default();
        hasher.write(bytes);
        self.hash = hasher.finish();
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

type TypeIdMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

struct SystemNode {
    parent: SystemId,
    entity: Option<Entity>,
    children: World,
}
