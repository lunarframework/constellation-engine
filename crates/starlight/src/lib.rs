//! `starlight` a lightweight, entity component relationship system

#![warn(missing_docs)]

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

mod archetype;
mod batch;
mod borrow;
mod bundle;
mod column;
mod command_buffer;
mod entities;
mod entity_builder;
mod entity_ref;
mod query;
mod query_one;
mod world;

pub use archetype::{Archetype, ArchetypeColumn};
pub use batch::{BatchIncomplete, BatchWriter, ColumnBatch, ColumnBatchBuilder, ColumnBatchType};
pub use bundle::{Bundle, DynamicBundle, MissingComponent};
pub use column::{Column, ColumnMut};
pub use command_buffer::CommandBuffer;
pub use entities::{Entity, NoSuchEntity};
pub use entity_builder::{BuiltEntity, BuiltEntityClone, EntityBuilder, EntityBuilderClone};
pub use entity_ref::{EntityRef, Ref, RefMut};
pub use query::{
    Access, Batch, BatchedIter, Fetch, Or, PreparedQuery, PreparedQueryBorrow, PreparedQueryIter,
    Query, QueryBorrow, QueryItem, QueryIter, QueryMut, Satisfies, With, Without,
};
pub use query_one::QueryOne;
pub use world::{
    ArchetypesGeneration, Component, ComponentError, Iter, QueryOneError, SpawnBatchIter,
    SpawnColumnBatchIter, World,
};

// // Unstable implementation details needed by the macros
// #[doc(hidden)]
// pub use archetype::TypeInfo;
// #[cfg(feature = "macros")]
// #[doc(hidden)]
// pub use lazy_static;
// #[doc(hidden)]
// pub use query::Fetch;

// #[cfg(feature = "macros")]
// pub use hecs_macros::{Bundle, Query};

fn align(x: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (x + alignment - 1) & (!alignment + 1)
}
