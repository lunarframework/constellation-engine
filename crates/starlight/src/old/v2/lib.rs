#![warn(missing_docs)]

//! A lightweight ECS
//!
//! This library essentially has the same goals as `hecs`, but includes additional functionality like native

/// Imagine macro parameters, but more like those Russian dolls.
///
/// Calls m!(A, B, C), m!(A, B), m!(B), and m!() for i.e. (m, A, B, C)
/// where m is any macro, for any number of parameters.
macro_rules! smaller_tuples_too {
    ($m: ident, $ty: ident) => {
        $m!{}
        $m!{$ty}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        smaller_tuples_too!{$m, $($tt),*}
        $m!{$ty, $($tt),*}
    };
}

fn align(x: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (x + alignment - 1) & (!alignment + 1)
}

mod archetype;
mod borrow;
mod entity;
mod world;

pub use archetype::{Archetype, ArchetypeSet};
pub use borrow::AtomicBorrow;
pub use entity::{
    Component, ComponentInfo, Components, Entities, Entity, EntityArg, EntityMeta, EntitySignature,
    NoSuchComponent, NoSuchEntity, Relation, ReserveEntitiesIterator,
};
pub use world::World;

use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use std::any::TypeId;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};

/// A HashMap with TypeId keys
///
/// Because TypeId is already a fully-hashed u64 (including data in the high seven bits,
/// which hashbrown needs), there is no need to hash it again. Instead, this uses the much
/// faster no-op hash.
type TypeIdMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

/// A hasher optimized for hashing a single TypeId.
///
/// TypeId is already thoroughly hashed, so there's no reason to hash it again.
/// Just leave the bits unchanged.
#[derive(Default)]
pub struct TypeIdHasher {
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
