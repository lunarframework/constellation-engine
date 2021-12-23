use crate::{
    ArchetypeSet, Component, Components, Entities, Entity, EntityMeta, ReserveEntitiesIterator,
    TypeIdMap,
};

use hashbrown::HashMap;
use spin::Mutex;

/// Generic ECS container
pub struct World {
    /// Map of registered components
    type_registry: TypeIdMap<Component>,
    /// Manges Entity ids and metadata
    entities: Entities,
    /// Manages component ids and metadata
    components: Components,
    /// Manages archetypes
    archetypes: ArchetypeSet,
    id: u64,
}

impl World {
    pub fn new() -> Self {
        // AtomicU64 is unsupported on 32-bit MIPS and PPC architectures
        // For compatibility, use Mutex<u64>
        static ID: Mutex<u64> = Mutex::new(1);
        let id = {
            let mut id = ID.lock();
            let next = id.checked_add(1).unwrap();
            *id = next;
            next
        };

        Self {
            type_registry: TypeIdMap::default(),
            entities: Entities::new(),
            components: Components::new(),
            archetypes: ArchetypeSet::new(),
            id,
        }
    }

    /// Allocate many entities ID concurrently
    ///
    /// Unlike [`spawn`](Self::spawn), this can be called concurrently with other operations on the
    /// [`World`] such as queries, but does not immediately create the entities. Reserved entities
    /// are not visible to queries or world iteration, but can be otherwise operated on
    /// freely. Operations that add or remove components or entities, such as `insert` or `despawn`,
    /// will cause all outstanding reserved entities to become real entities before proceeding. This
    /// can also be done explicitly by calling [`flush`](Self::flush).
    ///
    /// Useful for reserving an ID that will later have components attached to it with `insert`.
    pub fn reserve_entities(&self, count: u32) -> ReserveEntitiesIterator {
        self.entities.reserve_entities(count)
    }

    /// Allocate an entity ID concurrently
    ///
    /// See [`reserve_entities`](Self::reserve_entities).
    pub fn reserve_entity(&self) -> Entity {
        self.entities.reserve_entity()
    }

    /// Convert all reserved entities into empty entities that can be iterated and accessed
    ///
    /// Invoked implicitly by operations that add or remove components or entities, i.e. all
    /// variations of `spawn`, `despawn`, `insert`, and `remove`.
    pub fn flush(&mut self) {
        let arch = &mut self.archetypes.root();
        self.entities
            .flush(|id, location| location.index = unsafe { arch.allocate(id) });
    }

    /// Whether `entity` still exists
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }
}
