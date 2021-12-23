use crate::{
    AtomicBorrow, Component, ComponentInfo, Components, EntityArg, EntityMeta, EntitySignature,
};

use std::alloc::Layout;

/// Archetype, essentially a table of entities with identical components.
pub struct Archetype {
    signature: EntitySignatureWithMeta,

    columns: Box<[ArchetypeColumn]>,

    len: u32,
    entities: Box<[u32]>,
}

impl Archetype {
    pub(crate) fn new(signature: EntitySignatureWithMeta) -> Self {
        let count = signature.len();
        Self {
            columns: signature
                .iter()
                .map(|_arg| ArchetypeColumn::new())
                .collect::<Box<[_]>>(),
            signature,
            len: 0,
            entities: Box::new([]),
        }
    }

    pub(crate) fn clear(&mut self) {
        for (column, (arg, meta)) in self.columns.iter().zip(self.signature.args_with_meta()) {
            if let Some(info) = meta {
                for index in 0..self.len {
                    unsafe {
                        let removed = column.storage.add(index as usize * info.size() as usize);
                        info.drop(removed);
                    }
                }
            }
        }

        // Currently only plain old data types are supported
        self.len = 0;
    }

    pub(crate) fn borrow(&self, column: usize) {
        if !self.columns[column].state.borrow() {
            panic!("Column already borrowed uniquely");
        }
    }

    pub(crate) fn borrow_mut(&self, column: usize) {
        if !self.columns[column].state.borrow_mut() {
            panic!("Column already borrowed");
        }
    }

    pub(crate) fn release(&self, column: usize) {
        self.columns[column].state.release();
    }

    pub(crate) fn release_mut(&self, column: usize) {
        self.columns[column].state.release_mut();
    }

    /// Number of entities in this archetype
    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Whether this archetype contains no entities
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(crate) fn entities(&self) -> &[u32] {
        &self.entities
    }

    pub(crate) fn entity(&self, index: u32) -> u32 {
        self.entities[index as usize]
    }

    #[inline]
    pub(crate) fn set_entity(&mut self, index: usize, id: u32) {
        self.entities[index] = id;
    }

    pub fn signature(&self) -> &EntitySignature {
        &self.signature.signature
    }

    /// Every type must be written immediately after this call
    pub(crate) unsafe fn allocate(&mut self, id: u32) -> u32 {
        if self.len as usize == self.entities.len() {
            self.grow(64);
        }

        self.entities[self.len as usize] = id;
        self.len += 1;
        self.len - 1
    }

    pub(crate) unsafe fn set_len(&mut self, len: u32) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }

    pub(crate) fn reserve(&mut self, additional: u32) {
        if additional > (self.capacity() - self.len()) {
            let increment = additional - (self.capacity() - self.len());
            self.grow(increment.max(64));
        }
    }

    pub(crate) fn capacity(&self) -> u32 {
        self.entities.len() as u32
    }

    /// Increase capacity by at least `min_increment`
    fn grow(&mut self, min_increment: u32) {
        // Double capacity or increase it by `min_increment`, whichever is larger.
        self.grow_exact(self.capacity().max(min_increment))
    }

    /// Increase capacity by exactly `increment`
    fn grow_exact(&mut self, increment: u32) {
        unsafe {
            let old_count = self.len as usize;
            let old_cap = self.entities.len();
            let new_cap = self.entities.len() + increment as usize;
            let mut new_entities = vec![!0; new_cap].into_boxed_slice();
            new_entities[0..old_count].copy_from_slice(&self.entities[0..old_count]);
            self.entities = new_entities;

            let new_columns = self
                .columns
                .iter()
                .zip(self.signature.args_with_meta())
                .map(|(column, (arg, info))| {
                    let storage = if let Some(info) = info {
                        if info.size() == 0 {
                            std::ptr::null_mut()
                        } else {
                            let mem = std::alloc::alloc(
                                Layout::from_size_align(
                                    info.size() as usize * new_cap,
                                    info.align() as usize,
                                )
                                .unwrap(),
                            );
                            std::ptr::copy_nonoverlapping(
                                column.storage,
                                mem,
                                info.size() as usize * old_count,
                            );
                            if old_cap > 0 {
                                std::alloc::dealloc(
                                    column.storage,
                                    Layout::from_size_align(
                                        info.size() as usize * old_cap,
                                        info.align() as usize,
                                    )
                                    .unwrap(),
                                );
                            }
                            mem
                        }
                    } else {
                        std::ptr::null_mut()
                    };

                    ArchetypeColumn {
                        state: AtomicBorrow::new(), //&mut self ensures no outstanding borrows
                        storage: storage,
                    }
                })
                .collect::<Box<[_]>>();

            self.columns = new_columns;
        }
    }

    /// Returns the ID of the entity moved into `index`, if any
    pub(crate) unsafe fn remove(&mut self, index: u32, drop: bool) -> Option<u32> {
        assert!(index < self.len);
        let last = self.len - 1;
        for (column, (arg, info)) in self.columns.iter().zip(self.signature.args_with_meta()) {
            if let Some(info) = info {
                let removed = column.storage.add(index as usize * info.size() as usize);
                if drop {
                    info.drop(removed);
                }
                if index != last {
                    let moved = column.storage.add(last as usize * info.size() as usize);
                    std::ptr::copy_nonoverlapping(moved, removed, info.size() as usize);
                }
            }
        }
        self.len = last;
        if index != last {
            self.entities[index as usize] = self.entities[last as usize];
            Some(self.entities[last as usize])
        } else {
            None
        }
    }

    /// Returns the ID of the entity moved into `index`, if any
    pub(crate) unsafe fn move_to(
        &mut self,
        index: u32,
        mut f: impl FnMut(*mut u8, usize),
    ) -> Option<u32> {
        assert!(index < self.len);
        let last = self.len - 1;
        for (column, (arg, info)) in self.columns.iter().zip(self.signature.args_with_meta()) {
            if let Some(info) = info {
                let moved_out = column.storage.add((index * info.size()) as usize);
                f(moved_out, info.size() as usize);
                if index != last {
                    let moved = column.storage.add(last as usize * info.size() as usize);
                    std::ptr::copy_nonoverlapping(moved, moved_out, info.size() as usize);
                }
            }
        }
        self.len -= 1;
        if index != last {
            self.entities[index as usize] = self.entities[last as usize];
            Some(self.entities[last as usize])
        } else {
            None
        }
    }

    /// Add components from another archetype with identical components
    ///
    /// # Safety
    ///
    /// Component types must match exactly.
    pub(crate) unsafe fn merge(&mut self, mut other: Archetype) {
        assert_eq!(self.signature, other.signature);
        self.reserve(other.len);
        for (((arg, info), dst), src) in self
            .signature
            .args_with_meta()
            .zip(self.columns.iter())
            .zip(other.columns.iter())
        {
            if let Some(info) = info {
                dst.storage
                    .add(self.len as usize * info.size() as usize)
                    .copy_from_nonoverlapping(
                        src.storage,
                        other.len as usize * info.size() as usize,
                    )
            }
        }
        self.len += other.len;
        other.len = 0;
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        self.clear();
        if self.entities.len() == 0 {
            return;
        }
        for (column, (arg, info)) in self.columns.iter().zip(self.signature.args_with_meta()) {
            if let Some(info) = info {
                if info.size() != 0 {
                    unsafe {
                        std::alloc::dealloc(
                            column.storage,
                            std::alloc::Layout::from_size_align_unchecked(
                                info.size() as usize * self.entities.len(),
                                info.align() as usize,
                            ),
                        );
                    }
                }
            }
        }
    }
}

struct ArchetypeColumn {
    state: AtomicBorrow,
    storage: *mut u8,
}

impl ArchetypeColumn {
    fn new() -> Self {
        Self {
            state: AtomicBorrow::new(),
            storage: std::ptr::null_mut(),
        }
    }
}

use hashbrown::HashMap;

pub struct ArchetypeSet {
    index: HashMap<EntitySignature, u32>,
    archetypes: Vec<Archetype>,
    generation: u64,
    insert_edges: Vec<HashMap<EntityArg, InsertTarget>>,
    remove_edges: Vec<HashMap<EntityArg, RemoveTarget>>,
}

impl ArchetypeSet {
    pub(crate) fn new() -> Self {
        Self {
            index: Some((EntitySignature::empty(), 0)).into_iter().collect(),
            archetypes: vec![Archetype::new(EntitySignatureWithMeta::empty())],
            generation: 0,
            insert_edges: vec![HashMap::default()],
            remove_edges: vec![HashMap::default()],
        }
    }

    /// Find the archetype ID that has the given type
    /// Closure allows for lazy evaluation
    pub(crate) fn get<R: Iterator<Item = Option<ComponentInfo>>>(
        &mut self,
        signature: &EntitySignature,
        meta: impl FnOnce() -> R,
    ) -> u32 {
        self.index.get(signature).copied().unwrap_or_else(|| 0)
    }

    fn insert(
        &mut self,
        signature: &EntitySignature,
        meta: impl Iterator<Item = Option<ComponentInfo>>,
    ) -> u32 {
        let x = self.archetypes.len() as u32;
        let old = self.index.insert(signature.clone(), x);
        debug_assert!(old.is_none(), "inserted duplicate archetype");
        self.archetypes
            .push(Archetype::new(EntitySignatureWithMeta::new(
                signature.clone(),
                meta,
            )));
        self.post_insert();
        x
    }

    /// Returns archetype ID and starting location index
    fn insert_batch(&mut self, archetype: Archetype) -> (u32, u32) {
        use hashbrown::hash_map::Entry;

        let sig = archetype.signature().clone();

        match self.index.entry(sig) {
            Entry::Occupied(x) => {
                // Duplicate of existing archetype
                let existing = &mut self.archetypes[*x.get() as usize];
                let base = existing.len();
                unsafe {
                    existing.merge(archetype);
                }
                (*x.get(), base)
            }
            Entry::Vacant(x) => {
                // Brand new archetype
                let id = self.archetypes.len() as u32;
                self.archetypes.push(archetype);
                x.insert(id);
                self.post_insert();
                (id, 0)
            }
        }
    }

    fn post_insert(&mut self) {
        self.insert_edges.push(HashMap::default());
        self.remove_edges.push(HashMap::default());
        self.generation += 1;
    }

    pub fn root(&self) -> &Archetype {
        &self.archetypes[0]
    }

    pub fn root_mut(&mut self) -> &mut Archetype {
        &mut self.archetypes[0]
    }
}

/// Metadata cached for inserting components into entities from this archetype
struct InsertTarget {
    /// Components from the current archetype that are replaced by the insert
    replaced: Vec<EntityMeta>,
    /// Components from the current archetype that are moved by the insert
    retained: Vec<EntityMeta>,
    /// ID of the target archetype
    index: u32,
}

struct RemoveTarget {
    index: u32,
}

struct EntitySignatureWithMeta {
    signature: EntitySignature,
    meta: Box<[Option<ComponentInfo>]>,
}

impl EntitySignatureWithMeta {
    pub fn new(
        signature: EntitySignature,
        meta: impl IntoIterator<Item = Option<ComponentInfo>>,
    ) -> Self {
        Self {
            signature,
            meta: meta.into_iter().collect(),
        }
    }

    pub fn empty() -> Self {
        Self {
            signature: EntitySignature::empty(),
            meta: Box::new([]),
        }
    }

    pub fn len(&self) -> usize {
        self.signature.len()
    }

    pub fn has(&self, arg: EntityArg) -> bool {
        self.signature.has(arg)
    }

    pub fn arg(&self, index: u32) -> (EntityArg, Option<&ComponentInfo>) {
        (
            self.signature.arg(index),
            self.meta[index as usize].as_ref(),
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = EntityArg> + '_ {
        self.signature.iter().copied()
    }

    pub fn args_with_meta(&self) -> impl Iterator<Item = (EntityArg, Option<&ComponentInfo>)> {
        self.signature
            .iter()
            .copied()
            .zip(self.meta.iter().map(|m| m.as_ref()))
    }

    // pub fn comps_with_meta(&self) -> impl Iterator<Item = (EntityArg, &ComponentInfo)> {
    //     self.signature
    //         .iter()
    //         .copied()
    //         .zip(self.meta.iter().map(|m| m.as_ref()).flatten())
    // }
}
