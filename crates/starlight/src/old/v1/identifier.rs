use std::any::TypeId;
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

/// An Identifier for entities, components, or entity relation
/// This is a 64 bit integer, and information is stored in the following way
/// First 32 bits: entity or component
/// Next 31 bits: Meta data - eg generation, relation type, etc.
/// Topmost bit: Reserved as 1 so that Option<Identifier> takes up no extra space.
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityId {
    pub(crate) base: u32,
    pub(crate) meta: NonZeroU32,
}

impl EntityId {
    /// Convert to a form convenient for passing outside of rust
    ///
    /// No particular structure is guaranteed for the returned bits.
    ///
    /// Useful for storing entity IDs externally, or in conjunction with `Entity::from_bits` and
    /// `World::spawn_at` for easy serialization. Alternatively, consider `id` for more compact
    /// representation.
    pub fn to_bits(self) -> NonZeroU64 {
        unsafe {
            NonZeroU64::new_unchecked(u64::from(self.meta.get()) << 32 | u64::from(self.base))
        }
    }

    /// Reconstruct an `Entity` previously destructured with `to_bits` if the bitpattern is valid,
    /// else `None`
    ///
    /// Useful for storing entity IDs externally, or in conjunction with `Entity::to_bits` and
    /// `World::spawn_at` for easy serialization.
    pub fn from_bits(bits: u64) -> Option<Self> {
        Some(Self {
            meta: NonZeroU32::new((bits >> 32) as u32)?,
            base: bits as u32,
        })
    }
}

/// Lightweight unique ID, or handle, of an entity
///
/// Obtained from `World::spawn`. Can be stored to refer to an entity in the future.
///
/// Enable the `serde` feature on the crate to make this `Serialize`able. Some applications may be
/// able to save space by only serializing the output of `Entity::id`.
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entity(EntityId);

impl Entity {
    /// Creates new entity from id and generation count
    pub fn from_raw_parts(id: u32, gen: NonZeroU32) -> Self {
        Self(EntityId {
            base: id,
            meta: gen,
        })
    }

    /// Convert to a form convenient for passing outside of rust
    pub fn to_bits(self) -> NonZeroU64 {
        self.0.to_bits()
    }

    /// Reconstruct an `Entity` previously destructured with `to_bits` if the bitpattern is valid,
    /// else `None`
    pub fn from_bits(bits: u64) -> Option<Self> {
        Some(Self(EntityId::from_bits(bits)?))
    }

    /// Extract a transiently unique identifier
    ///
    /// No two simultaneously-live entities share the same ID, but dead entities' IDs may collide
    /// with both live and dead entities. Useful for compactly representing entities within a
    /// specific snapshot of the world, such as when serializing.
    ///
    /// See also `World::find_entity_from_id`.
    pub fn id(self) -> u32 {
        self.0.base
    }

    /// Returns the generation of the entity
    pub fn generation(self) -> NonZeroU32 {
        self.0.meta
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.id(), self.generation())
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

use std::alloc::Layout;

pub struct Type(Vec<EntityId>);

/// Metadata required to store a component.
///
/// All told, this means a [`TypeId`], to be able to dynamically name/check the component type; a
/// [`Layout`], so that we know how to allocate memory for this component type; and a drop function
/// which internally calls [`core::ptr::drop_in_place`] with the correct type parameter.
#[derive(Debug, Copy, Clone)]
pub struct TypeInfo {
    pub(crate) id: TypeId,
    pub(crate) layout: Layout,
    pub(crate) drop: unsafe fn(*mut u8),
    pub(crate) type_name: &'static str,
}

impl TypeInfo {
    /// Construct a `TypeInfo` directly from the static type.
    pub fn of<T: 'static>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }

        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
            #[cfg(debug_assertions)]
            type_name: core::any::type_name::<T>(),
        }
    }

    /// Construct a `TypeInfo` from its components. This is useful in the rare case that you have
    /// some kind of pointer to raw bytes/erased memory holding a component type, coming from a
    /// source unrelated to hecs, and you want to treat it as an insertable component by
    /// implementing the `DynamicBundle` API.
    pub fn from_parts(id: TypeId, layout: Layout, drop: unsafe fn(*mut u8)) -> Self {
        Self {
            id,
            layout,
            drop,
            #[cfg(debug_assertions)]
            type_name: "<unknown> (TypeInfo constructed from parts)",
        }
    }

    /// Access the `TypeId` for this component type.
    pub fn id(&self) -> TypeId {
        self.id
    }

    /// Access the `Layout` of this component type.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Directly call the destructor on a pointer to data of this component type.
    ///
    /// # Safety
    ///
    /// All of the caveats of [`core::ptr::drop_in_place`] apply, with the additional requirement
    /// that this method is being called on a pointer to an object of the correct component type.
    pub unsafe fn drop(&self, data: *mut u8) {
        (self.drop)(data)
    }

    /// Get the function pointer encoding the destructor for the component type this `TypeInfo`
    /// represents.
    pub fn drop_shim(&self) -> unsafe fn(*mut u8) {
        self.drop
    }
}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeInfo {
    /// Order by alignment, descending. Ties broken with TypeId.
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.layout
            .align()
            .cmp(&other.layout.align())
            .reverse()
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeInfo {}
