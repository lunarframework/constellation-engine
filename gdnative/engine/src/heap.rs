use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use std::alloc::Layout;
use std::any::{Any, TypeId};
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeapError {
    #[error("The pointer {0:?} was invalid")]
    InvalidPointer(HeapPointer),
    #[error("The type with id {0:?} was used but not yet registered")]
    UnregisteredType(TypeId),
}

pub struct Heap {
    vecs: TypeIdMap<HeapVec>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            vecs: TypeIdMap::default(),
        }
    }

    pub fn register<T: Any>(&mut self) {
        self.vecs
            .entry(TypeId::of::<T>())
            .or_insert_with(|| HeapVec::new::<T>());
    }

    pub fn insert<T: Any>(&mut self, element: T) -> Result<HeapPointer, HeapError> {
        let type_id = TypeId::of::<T>();
        match self.vecs.get_mut(&type_id) {
            Some(vec) => Ok(vec.insert(element)),
            None => Err(HeapError::UnregisteredType(type_id)),
        }
    }

    pub fn remove(&mut self, ptr: HeapPointer) -> Result<(), HeapError> {
        match self.vecs.get_mut(&ptr.1) {
            Some(vec) => {
                vec.remove(ptr)?;
                Ok(())
            }
            None => Err(HeapError::UnregisteredType(ptr.1)),
        }
    }

    pub fn iter<T: Any>(&self) -> Result<Iter<'_, T>, HeapError> {
        let type_id = TypeId::of::<T>();
        match self.vecs.get(&type_id) {
            Some(vec) => Ok(vec.iter::<T>()),
            None => Err(HeapError::UnregisteredType(type_id)),
        }
    }

    pub fn iter_mut<T: Any>(&mut self) -> Result<IterMut<'_, T>, HeapError> {
        let type_id = TypeId::of::<T>();
        match self.vecs.get_mut(&type_id) {
            Some(vec) => Ok(vec.iter_mut::<T>()),
            None => Err(HeapError::UnregisteredType(type_id)),
        }
    }

    pub fn get<T: Any>(&self, ptr: HeapPointer) -> Result<&T, HeapError> {
        let type_id = TypeId::of::<T>();
        match self.vecs.get(&type_id) {
            Some(vec) => Ok(vec.get::<T>(ptr)?),
            None => Err(HeapError::UnregisteredType(type_id)),
        }
    }

    pub fn get_mut<T: Any>(&mut self, ptr: HeapPointer) -> Result<&mut T, HeapError> {
        let type_id = TypeId::of::<T>();
        match self.vecs.get_mut(&type_id) {
            Some(vec) => Ok(vec.get_mut::<T>(ptr)?),
            None => Err(HeapError::UnregisteredType(type_id)),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct HeapPointer(usize, TypeId);

struct HeapVec {
    type_id: TypeId,
    type_layout: Layout,
    type_drop: unsafe fn(*mut u8),
    count: usize,
    elements: *mut u8,
    used: Vec<bool>,
    next_free: Option<usize>,
}

impl HeapVec {
    pub fn new<T: Any>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }

        Self {
            type_id: TypeId::of::<T>(),
            type_layout: Layout::new::<T>().pad_to_align(),
            type_drop: drop_ptr::<T>,
            count: 0,
            elements: std::ptr::null_mut::<u8>(),
            used: Vec::new(),
            next_free: None,
        }
    }

    pub fn insert<T: Any>(&mut self, data: T) -> HeapPointer {
        assert!(self.type_id == TypeId::of::<T>());

        let index = self.next_free();

        unsafe {
            let dst = self
                .elements
                .add(index * self.type_layout.size())
                .cast::<T>();

            std::ptr::copy_nonoverlapping(&data, dst, 1);
        }

        HeapPointer(index, self.type_id)
    }

    pub fn get<T: Any>(&self, ptr: HeapPointer) -> Result<&T, HeapError> {
        assert!(self.type_id == TypeId::of::<T>());

        if !self.is_valid(ptr) {
            return Err(HeapError::InvalidPointer(ptr));
        }

        Ok(unsafe {
            self.elements
                .add(ptr.0 * self.type_layout.size())
                .cast::<T>()
                .as_ref()
                .unwrap()
        })
    }

    pub fn get_mut<T: Any>(&mut self, ptr: HeapPointer) -> Result<&mut T, HeapError> {
        assert!(self.type_id == TypeId::of::<T>());

        if !self.is_valid(ptr) {
            return Err(HeapError::InvalidPointer(ptr));
        }

        Ok(unsafe {
            self.elements
                .add(ptr.0 * self.type_layout.size())
                .cast::<T>()
                .as_mut()
                .unwrap()
        })
    }

    pub fn iter<T: Any>(&self) -> Iter<'_, T> {
        Iter {
            vec: self,
            index: 0,
            _marker: std::marker::PhantomData::<T>,
        }
    }

    pub fn iter_mut<T: Any>(&mut self) -> IterMut<'_, T> {
        IterMut {
            vec: self,
            index: 0,
            _marker: std::marker::PhantomData::<T>,
        }
    }

    pub fn remove(&mut self, ptr: HeapPointer) -> Result<(), HeapError> {
        if !self.is_valid(ptr) {
            return Err(HeapError::InvalidPointer(ptr));
        }

        unsafe {
            let data = self.elements.add(ptr.0 * self.type_layout.size());

            (self.type_drop)(data);
        }

        self.used[ptr.0] = false;

        if self.next_free.is_some() {
            if ptr.0 < self.next_free.unwrap() {
                self.next_free = Some(ptr.0);
            }
        } else {
            self.next_free = Some(ptr.0);
        };

        Ok(())
    }

    pub fn clear(&mut self) {
        for i in 0..self.count {
            if self.used[i] {
                unsafe {
                    (self.type_drop)(self.elements.add(i * self.type_layout.size()));
                }
            }
        }

        self.used.fill(false);

        if self.count > 0 {
            self.next_free = Some(0);
        } else {
            self.next_free = None;
        }
    }

    pub fn is_valid(&self, ptr: HeapPointer) -> bool {
        if self.count > ptr.0 && ptr.1 == self.type_id {
            self.used[ptr.0]
        } else {
            false
        }
    }

    fn grow(&mut self, min_increment: usize) {
        self.grow_exact(self.count.max(min_increment));
    }

    fn grow_exact(&mut self, increment: usize) {
        let old_elements = self.elements;
        let old_count = self.count;
        let new_count = old_count + increment;
        unsafe {
            let new_elements = std::alloc::alloc(
                Layout::from_size_align(
                    self.type_layout.size() * new_count,
                    self.type_layout.align(),
                )
                .unwrap(),
            );

            if !old_elements.is_null() {
                std::ptr::copy_nonoverlapping(
                    old_elements,
                    new_elements,
                    self.type_layout.size() * old_count,
                );

                std::alloc::dealloc(
                    old_elements,
                    Layout::from_size_align(
                        self.type_layout.size() * old_count,
                        self.type_layout.align(),
                    )
                    .unwrap(),
                );
            }
            self.elements = new_elements;
            self.count = new_count;
        }

        self.used.resize(new_count, false);
        if self.next_free.is_none() {
            self.next_free = Some(old_count);
        }
    }

    fn next_free(&mut self) -> usize {
        if self.next_free.is_none() {
            self.grow(16);
        }

        let index = self.next_free.take().unwrap();

        self.used[index] = true;

        for i in (index + 1)..self.count {
            if !self.used[index] {
                self.next_free = Some(i);
            }
        }

        index
    }
}

impl Drop for HeapVec {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, T: Any> {
    vec: &'a HeapVec,
    index: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Any> Iterator for Iter<'a, T> {
    type Item = (HeapPointer, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.vec.count {
            let index = self.index;
            self.index += 1;

            if self.vec.used[index] {
                return Some((HeapPointer(index, self.vec.type_id), unsafe {
                    self.vec
                        .elements
                        .add(index * self.vec.type_layout.size())
                        .cast::<T>()
                        .as_ref()
                        .unwrap()
                }));
            }
        }
        None
    }
}

pub struct IterMut<'a, T: Any> {
    vec: &'a mut HeapVec,
    index: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Any> Iterator for IterMut<'a, T> {
    type Item = (HeapPointer, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.vec.count {
            let index = self.index;
            self.index += 1;

            if self.vec.used[index] {
                return Some((HeapPointer(index, self.vec.type_id), unsafe {
                    self.vec
                        .elements
                        .add(index * self.vec.type_layout.size())
                        .cast::<T>()
                        .as_mut()
                        .unwrap()
                }));
            }
        }
        None
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
