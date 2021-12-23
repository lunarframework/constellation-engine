//! Includes all entity_id related functionality.
//! Because `starlight` considers components to simply be a type
//! of entity, this includes meta-type info.

use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicIsize, Ordering};

/// Lightweight handle to entity
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    id: u32,
    gen: NonZeroU32,
}

impl Entity {
    /// Constructs entity from id and generation could
    pub fn from_raw_parts(id: u32, gen: NonZeroU32) -> Self {
        Self { id: id, gen }
    }

    /// Constructs entity from bits
    pub fn from_bits(bits: u64) -> Option<Self> {
        Some(Self {
            id: bits as u32,
            gen: NonZeroU32::new((bits >> 32) as u32)?,
        })
    }

    /// Returns handle as bits
    pub fn to_bits(self) -> u64 {
        u64::from(self.gen.get()) << 32 | u64::from(self.id)
    }

    /// Returns the id of the entity
    pub fn id(self) -> u32 {
        self.id
    }

    /// Returns the generation of the entity
    pub fn gen(self) -> NonZeroU32 {
        self.gen
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.id(), self.gen())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Entity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_bits().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Entity, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;

        match Entity::from_bits(bits) {
            Some(ent) => Ok(ent),
            None => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(bits),
                &"`a valid `Entity` bitpattern",
            )),
        }
    }
}

// impl EntityId {
//     /// Constructs a new
//     pub fn from_raw_parts(id: u32, meta: u16, flags: u16) -> Self {
//         Self(u64::from(id) | (u64::from(meta) << 32) | (u64::from(flags) << 48))
//     }

//     /// Returns the lowest 32 bits, ie the base id
//     pub fn id(self) -> u32 {
//         self.0 as u32
//     }

//     /// Returns the meta data as a u16.
//     pub fn meta(self) -> u16 {
//         (self.0 >> 32) as u16
//     }

//     /// Returns the flags stored in the entity. 0x00 means default usage.
//     /// These only have meaning within archetype storage (for example indicating
//     /// that an entity id is a component, or is disableable)
//     /// archetype storage, and thus should be ignored otherwise.
//     pub fn flags(self) -> u16 {
//         (self.0 >> 48) as u16
//     }
// }

/// Structure detailing extra information required to store an entity
#[derive(Clone, Copy)]
pub struct EntityMeta {
    /// Location of the entity
    pub location: Location,
    /// Generation number
    pub generation: NonZeroU32,
}

impl EntityMeta {
    const EMPTY: EntityMeta = EntityMeta {
        generation: match NonZeroU32::new(1) {
            Some(x) => x,
            None => unreachable!(),
        },
        location: Location {
            archetype: 0,
            index: u32::max_value(), // dummy value, to be filled in
        },
    };
}

// impl PartialOrd for EntityMeta {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for EntityMeta {
//     /// Order by alignment, descending. Ties broken with TypeId.
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.id.cmp(&other.id)
//     }
// }

// impl PartialEq for EntityMeta {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl Eq for EntityMeta {}

/// Specifies the location in the archetype graph of the entity
#[derive(Clone, Copy)]
pub struct Location {
    /// The Archtype the entity is currently stored in
    pub archetype: u32,
    /// The row the entity is in
    pub index: u32,
}

/// An iterator returning a sequence of Entity values from `Entities::reserve_entities`.
pub struct ReserveEntitiesIterator<'a> {
    // Metas, so we can recover the current generation for anything in the freelist.
    meta: &'a [EntityMeta],

    // Reserved IDs formerly in the freelist to hand out.
    id_iter: std::slice::Iter<'a, u32>,

    // New Entity IDs to hand out, outside the range of meta.len().
    id_range: std::ops::Range<u32>,
}

impl<'a> Iterator for ReserveEntitiesIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.id_iter
            .next()
            .map(|&id| Entity::from_raw_parts(id, self.meta[id as usize].generation))
            .or_else(|| {
                self.id_range
                    .next()
                    .map(|id| Entity::from_raw_parts(id, NonZeroU32::new(1).unwrap()))
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.id_iter.len() + self.id_range.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ReserveEntitiesIterator<'a> {}

/// Object in charge of allocating and managing EntityIds
pub struct Entities {
    meta: Vec<EntityMeta>,

    // The `pending` and `free_cursor` fields describe three sets of Entity IDs
    // that have been freed or are in the process of being allocated:
    //
    // - The `freelist` IDs, previously freed by `free()`. These IDs are available to any
    //   of `alloc()`, `reserve_entity()` or `reserve_entities()`. Allocation will
    //   always prefer these over brand new IDs.
    //
    // - The `reserved` list of IDs that were once in the freelist, but got
    //   reserved by `reserve_entities` or `reserve_entity()`. They are now waiting
    //   for `flush()` to make them fully allocated.
    //
    // - The count of new IDs that do not yet exist in `self.meta()`, but which
    //   we have handed out and reserved. `flush()` will allocate room for them in `self.meta()`.
    //
    // The contents of `pending` look like this:
    //
    // ```
    // ----------------------------
    // |  freelist  |  reserved   |
    // ----------------------------
    //              ^             ^
    //          free_cursor   pending.len()
    // ```
    //
    // As IDs are allocated, `free_cursor` is atomically decremented, moving
    // items from the freelist into the reserved list by sliding over the boundary.
    //
    // Once the freelist runs out, `free_cursor` starts going negative.
    // The more negative it is, the more IDs have been reserved starting exactly at
    // the end of `meta.len()`.
    //
    // This formulation allows us to reserve any number of IDs first from the freelist
    // and then from the new IDs, using only a single atomic subtract.
    //
    // Once `flush()` is done, `free_cursor` will equal `pending.len()`.
    pending: Vec<u32>,
    free_cursor: AtomicIsize,
    len: u32,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            free_cursor: AtomicIsize::new(0),
            len: 0,
            meta: Vec::default(),
            pending: Vec::default(),
        }
    }
    /// Reserve entity IDs concurrently
    ///
    /// Storage for entity generation and location is lazily allocated by calling `flush`.
    pub fn reserve_entities(&self, count: u32) -> ReserveEntitiesIterator {
        // Use one atomic subtract to grab a range of new IDs. The range might be
        // entirely nonnegative, meaning all IDs come from the freelist, or entirely
        // negative, meaning they are all new IDs to allocate, or a mix of both.
        let range_end = self
            .free_cursor
            .fetch_sub(count as isize, Ordering::Relaxed);
        let range_start = range_end - count as isize;

        let freelist_range = range_start.max(0) as usize..range_end.max(0) as usize;

        let (new_id_start, new_id_end) = if range_start >= 0 {
            // We satisfied all requests from the freelist.
            (0, 0)
        } else {
            // We need to allocate some new Entity IDs outside of the range of self.meta.
            //
            // `range_start` covers some negative territory, e.g. `-3..6`.
            // Since the nonnegative values `0..6` are handled by the freelist, that
            // means we need to handle the negative range here.
            //
            // In this example, we truncate the end to 0, leaving us with `-3..0`.
            // Then we negate these values to indicate how far beyond the end of `meta.end()`
            // to go, yielding `meta.len()+0 .. meta.len()+3`.
            let base = self.meta.len() as isize;

            let new_id_end = u32::try_from(base - range_start).expect("too many entities");

            // `new_id_end` is in range, so no need to check `start`.
            let new_id_start = (base - range_end.min(0)) as u32;

            (new_id_start, new_id_end)
        };

        ReserveEntitiesIterator {
            meta: &self.meta[..],
            id_iter: self.pending[freelist_range].iter(),
            id_range: new_id_start..new_id_end,
        }
    }

    /// Reserve one entity ID concurrently
    ///
    /// Equivalent to `self.reserve_entities(1).next().unwrap()`, but more efficient.
    pub fn reserve_entity(&self) -> Entity {
        let n = self.free_cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            // Allocate from the freelist.
            let id = self.pending[(n - 1) as usize];
            Entity::from_raw_parts(id, self.meta[id as usize].generation)
        } else {
            // Grab a new ID, outside the range of `meta.len()`. `flush()` must
            // eventually be called to make it valid.
            //
            // As `self.free_cursor` goes more and more negative, we return IDs farther
            // and farther beyond `meta.len()`.
            Entity::from_raw_parts(
                u32::try_from(self.meta.len() as isize - n).expect("too many entities"),
                NonZeroU32::new(1).unwrap(),
            )
        }
    }

    /// Check that we do not have pending work requiring `flush()` to be called.
    fn verify_flushed(&mut self) {
        debug_assert!(
            !self.needs_flush(),
            "flush() needs to be called before this operation is legal"
        );
    }

    /// Allocate an entity ID directly
    ///
    /// Location should be written immediately.
    pub fn alloc(&mut self) -> Entity {
        self.verify_flushed();

        self.len += 1;
        if let Some(id) = self.pending.pop() {
            let new_free_cursor = self.pending.len() as isize;
            self.free_cursor.store(new_free_cursor, Ordering::Relaxed); // Not racey due to &mut self
            Entity::from_raw_parts(id, self.meta[id as usize].generation)
        } else {
            let id = u32::try_from(self.meta.len()).expect("too many entities");
            self.meta.push(EntityMeta::EMPTY);
            Entity::from_raw_parts(id, NonZeroU32::new(1).unwrap())
        }
    }

    /// Allocate and set locations for many entity IDs laid out contiguously in an archetype
    ///
    /// `self.finish_alloc_many()` must be called after!
    pub fn alloc_many(&mut self, n: u32, archetype: u32, mut first_index: u32) -> AllocManyState {
        self.verify_flushed();

        let fresh = (n as usize).saturating_sub(self.pending.len()) as u32;
        assert!(
            (self.meta.len() + fresh as usize) < u32::MAX as usize,
            "too many entities"
        );
        let pending_end = self.pending.len().saturating_sub(n as usize);
        for &id in &self.pending[pending_end..] {
            self.meta[id as usize].location = Location {
                archetype,
                index: first_index,
            };
            first_index += 1;
        }

        let fresh_start = self.meta.len() as u32;
        self.meta.extend(
            (first_index..(first_index + fresh)).map(|index| EntityMeta {
                generation: NonZeroU32::new(1).unwrap(),
                location: Location { archetype, index },
            }),
        );

        self.len += n;

        AllocManyState {
            fresh: fresh_start..(fresh_start + fresh),
            pending_end,
        }
    }

    /// Remove entities used by `alloc_many` from the freelist
    ///
    /// This is an awkward separate function to avoid borrowck issues in `SpawnColumnBatchIter`.
    pub fn finish_alloc_many(&mut self, pending_end: usize) {
        self.pending.truncate(pending_end);
    }

    /// Allocate a specific entity ID, overwriting its generation
    ///
    /// Returns the location of the entity currently using the given ID, if any. Location should be written immediately.
    pub fn alloc_at(&mut self, entity: Entity) -> Option<Location> {
        self.verify_flushed();

        let loc = if entity.id() as usize >= self.meta.len() {
            self.pending.extend((self.meta.len() as u32)..entity.id());
            let new_free_cursor = self.pending.len() as isize;
            self.free_cursor.store(new_free_cursor, Ordering::Relaxed); // Not racey due to &mut self
            self.meta
                .resize(entity.id() as usize + 1, EntityMeta::EMPTY);
            self.len += 1;
            None
        } else if let Some(index) = self.pending.iter().position(|item| *item == entity.id()) {
            self.pending.swap_remove(index);
            let new_free_cursor = self.pending.len() as isize;
            self.free_cursor.store(new_free_cursor, Ordering::Relaxed); // Not racey due to &mut self
            self.len += 1;
            None
        } else {
            Some(std::mem::replace(
                &mut self.meta[entity.id() as usize].location,
                EntityMeta::EMPTY.location,
            ))
        };

        self.meta[entity.id() as usize].generation = entity.gen();

        loc
    }

    /// Destroy an entity, allowing it to be reused
    ///
    /// Must not be called while reserved entities are awaiting `flush()`.
    pub fn free(&mut self, entity: Entity) -> Result<Location, NoSuchEntity> {
        self.verify_flushed();

        let meta = self
            .meta
            .get_mut(entity.id() as usize)
            .ok_or(NoSuchEntity)?;
        if meta.generation != entity.gen() {
            return Err(NoSuchEntity);
        }

        meta.generation = NonZeroU32::new(u32::from(meta.generation).wrapping_add(1))
            .unwrap_or_else(|| NonZeroU32::new(1).unwrap());

        let loc = std::mem::replace(&mut meta.location, EntityMeta::EMPTY.location);

        self.pending.push(entity.id());

        let new_free_cursor = self.pending.len() as isize;
        self.free_cursor.store(new_free_cursor, Ordering::Relaxed); // Not racey due to &mut self
        self.len -= 1;

        Ok(loc)
    }

    /// Ensure at least `n` allocations can succeed without reallocating
    pub fn reserve(&mut self, additional: u32) {
        self.verify_flushed();

        let freelist_size = self.free_cursor.load(Ordering::Relaxed);
        let shortfall = additional as isize - freelist_size;
        if shortfall > 0 {
            self.meta.reserve(shortfall as usize);
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        // Note that out-of-range IDs are considered to be "contained" because
        // they must be reserved IDs that we haven't flushed yet.
        self.meta
            .get(entity.id() as usize)
            .map_or(true, |meta| meta.generation == entity.gen())
    }

    pub fn clear(&mut self) {
        self.meta.clear();
        self.pending.clear();
        self.free_cursor.store(0, Ordering::Relaxed); // Not racey due to &mut self
    }

    /// Access the location storage of an entity
    ///
    /// Must not be called on pending entities.
    pub fn get_mut(&mut self, entity: Entity) -> Result<&mut Location, NoSuchEntity> {
        let meta = self
            .meta
            .get_mut(entity.id() as usize)
            .ok_or(NoSuchEntity)?;
        if meta.generation == entity.gen() {
            Ok(&mut meta.location)
        } else {
            Err(NoSuchEntity)
        }
    }

    /// Returns `Ok(Location { archetype: 0, index: undefined })` for pending entities
    pub fn get(&self, entity: Entity) -> Result<Location, NoSuchEntity> {
        if self.meta.len() <= entity.id() as usize {
            return Ok(Location {
                archetype: 0,
                index: u32::max_value(),
            });
        }
        let meta = &self.meta[entity.id() as usize];
        if meta.generation != entity.gen() {
            return Err(NoSuchEntity);
        }
        Ok(meta.location)
    }

    /// Panics if the given id would represent an index outside of `meta`.
    ///
    /// # Safety
    /// Must only be called for currently allocated `id`s.
    pub unsafe fn resolve_unknown_gen(&self, id: u32) -> Entity {
        let meta_len = self.meta.len();

        if meta_len > id as usize {
            Entity::from_raw_parts(id, self.meta[id as usize].generation)
        } else {
            // See if it's pending, but not yet flushed.
            let free_cursor = self.free_cursor.load(Ordering::Relaxed);
            let num_pending = std::cmp::max(-free_cursor, 0) as usize;

            if meta_len + num_pending > id as usize {
                // Pending entities will have generation 0.
                Entity::from_raw_parts(id, NonZeroU32::new(1).unwrap())
            } else {
                panic!("entity id is out of range");
            }
        }
    }

    fn needs_flush(&mut self) -> bool {
        // Not racey due to &mut self
        self.free_cursor.load(Ordering::Relaxed) != self.pending.len() as isize
    }

    /// Allocates space for entities previously reserved with `reserve_entity` or
    /// `reserve_entities`, then initializes each one using the supplied function.
    pub fn flush(&mut self, mut init: impl FnMut(u32, &mut Location)) {
        // Not racey due because of self is &mut.
        let free_cursor = self.free_cursor.load(Ordering::Relaxed);

        let new_free_cursor = if free_cursor >= 0 {
            free_cursor as usize
        } else {
            let old_meta_len = self.meta.len();
            let new_meta_len = old_meta_len + -free_cursor as usize;
            self.meta.resize(new_meta_len, EntityMeta::EMPTY);

            self.len += -free_cursor as u32;
            for (id, meta) in self.meta.iter_mut().enumerate().skip(old_meta_len) {
                init(id as u32, &mut meta.location);
            }

            self.free_cursor.store(0, Ordering::Relaxed);
            0
        };

        self.len += (self.pending.len() - new_free_cursor) as u32;
        for id in self.pending.drain(new_free_cursor..) {
            init(id, &mut self.meta[id as usize].location);
        }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Error indicating that no entity with a particular ID exists
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchEntity;

impl fmt::Display for NoSuchEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("no such entity")
    }
}

impl Error for NoSuchEntity {}

#[derive(Clone)]
pub(crate) struct AllocManyState {
    pub pending_end: usize,
    fresh: std::ops::Range<u32>,
}

impl AllocManyState {
    pub fn next(&mut self, entities: &Entities) -> Option<u32> {
        if self.pending_end < entities.pending.len() {
            let id = entities.pending[self.pending_end];
            self.pending_end += 1;
            Some(id)
        } else {
            self.fresh.next()
        }
    }

    pub fn len(&self, entities: &Entities) -> usize {
        self.fresh.len() + (entities.pending.len() - self.pending_end)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Component(u32);

impl Component {
    /// Constructs entity from id and generation could
    pub fn from_raw_parts(id: u32) -> Self {
        Self(id)
    }

    /// Returns the id of the component
    pub fn id(self) -> u32 {
        self.0
    }
}

/// Extra information required to store components
#[derive(Clone)]
pub struct ComponentInfo {
    size: u32,
    align: u32,
    name: &'static str,
    drop: unsafe fn(*mut u8),
}

impl ComponentInfo {
    pub fn of<T: 'static>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }

        Self {
            size: std::mem::size_of::<T>() as u32,
            align: std::mem::align_of::<T>() as u32,
            name: std::any::type_name::<T>(),
            drop: drop_ptr::<T>,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn align(&self) -> u32 {
        self.align
    }

    pub unsafe fn drop(&self, data: *mut u8) {
        (self.drop)(data)
    }

    pub unsafe fn name(&self) -> &'static str {
        self.name
    }
}

pub struct Components {
    info: Vec<Option<ComponentInfo>>,
    next_id: u32,
    freelist: Vec<u32>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            info: Vec::default(),
            next_id: 0,
            freelist: Vec::default(),
        }
    }

    pub fn register(&mut self, info: ComponentInfo) -> Component {
        if let Some(id) = self.freelist.pop() {
            self.info[id as usize] = Some(info);
            return Component(id);
        }

        let id = self.next_id;
        self.next_id += 1;
        self.info.push(Some(info));
        Component(id)
    }

    pub fn unregister(&mut self, component: Component) {
        self.info[component.id() as usize].take();
        self.freelist.push(component.id());
    }

    pub fn get(&self, component: Component) -> Result<&ComponentInfo, NoSuchComponent> {
        let info = self
            .info
            .get(component.id() as usize)
            .ok_or(NoSuchComponent)?;
        match info {
            Some(info) => Ok(info),
            None => Err(NoSuchComponent),
        }
    }
}

/// Error indicating that no entity with a particular ID exists
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchComponent;

impl fmt::Display for NoSuchComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("no such component")
    }
}

impl Error for NoSuchComponent {}

/// Represents a relation
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Relation(u16);

impl Relation {
    /// NULL relation. Indicates that this argument is simply a component.
    pub const NULL: Relation = Relation(0x0000);

    /// Creates new relation from id
    pub fn from_raw_parts(id: u16) -> Self {
        Self(id)
    }

    /// The 16-bit ID of the relation
    pub fn id(self) -> u16 {
        self.0
    }
}

/// Represents a part of an EntitySignature.
/// Can either store a component or a relationship.
#[derive(Clone, Copy, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct EntityArg(u64);

impl EntityArg {
    /// Constructs the argument from an id and a relation
    pub fn from_raw_parts(id: u32, relation: u16) -> Self {
        Self(u64::from(id) | (u64::from(relation) << 32))
    }

    /// Returns the id stored in the argument
    pub fn id(self) -> u32 {
        self.0 as u32
    }

    /// Stores the relation stored in the argument
    pub fn relation(self) -> u16 {
        (self.0 >> 32) as u16
    }

    /// Checks whether this is a component
    pub fn is_component(self) -> bool {
        self.relation() == Relation::NULL.id()
    }

    /// Checks whether this is a relationship
    pub fn is_relationship(self) -> bool {
        !self.is_component()
    }

    // pub fn flags(self) -> u16 {
    //     (self.0 >> 48) as u16
    // }
}

/// Stores the `type` of an entity, ie the arguments added to it.
#[derive(Clone)]
pub struct EntitySignature {
    args: Box<[EntityArg]>,
    index: OrderedEntityArgMap<usize>,
}

impl EntitySignature {
    pub fn new(args: impl IntoIterator<Item = EntityArg>) -> Self {
        let mut args = args.into_iter().collect::<Box<[_]>>();
        args.sort_unstable();
        let index = OrderedEntityArgMap::new(args.iter().enumerate().map(|(i, &arg)| (arg, i)));
        Self { args, index }
    }

    pub fn empty() -> Self {
        Self {
            args: Box::new([]),
            index: OrderedEntityArgMap::empty(),
        }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn has(&self, arg: EntityArg) -> bool {
        self.index.contains_key(arg)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EntityArg> {
        self.args.iter()
    }

    pub fn arg(&self, index: u32) -> EntityArg {
        self.args[index as usize]
    }
}

impl PartialEq for EntitySignature {
    fn eq(&self, other: &Self) -> bool {
        let equal = true;
        for (arg, other) in self.args.iter().zip(other.args.iter()) {
            equal &= (arg == other);
        }
        equal
    }
}

impl Eq for EntitySignature {}

impl Hash for EntitySignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for arg in self.args.iter() {
            arg.hash(state);
        }
    }
}

#[derive(Clone)]
struct OrderedEntityArgMap<V>(Box<[(EntityArg, V)]>);

impl<V> OrderedEntityArgMap<V> {
    pub fn new(iter: impl IntoIterator<Item = (EntityArg, V)>) -> Self {
        let mut vals = iter.into_iter().collect::<Box<[_]>>();
        vals.sort_unstable_by_key(|(id, _)| *id);
        Self(vals)
    }

    pub fn empty() -> Self {
        Self(Box::new([]))
    }

    pub fn search(&self, arg: EntityArg) -> Option<usize> {
        self.0.binary_search_by_key(&arg, |(id, _)| *id).ok()
    }

    pub fn contains_key(&self, arg: EntityArg) -> bool {
        self.search(arg).is_some()
    }

    pub fn get(&self, arg: EntityArg) -> Option<&V> {
        self.search(arg).map(move |idx| &self.0[idx].1)
    }
}
