use super::{Root, System, SystemNode};
use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::{Any, TypeId};
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::marker::PhantomData;

pub trait Config: Any + Send + Sync {}

#[derive(Serialize, Deserialize)]
pub struct SystemTree<R: System + Root> {
    root: SystemNode<R>,
    config: SystemConfigWrapper<R>,
}

impl<R: System + Root> SystemTree<R> {
    pub fn new(root: R) -> Self {
        Self {
            root: SystemNode::new(root),
            config: SystemConfigWrapper(R::default_config(), PhantomData),
        }
    }

    pub fn solve(&mut self, start: f64, end: f64, iterations: usize) {
        self.root.solve_begin(&self.config.0, start);

        let mut time = start;
        let delta = (end - start) / (iterations + 1) as f64;

        for _i in 0..(iterations + 1) {
            self.root.solve_update(&self.config.0, time, delta);
            time += delta;
        }

        self.root.solve_end(&self.config.0, time);
    }

    pub fn root(&self) -> &SystemNode<R> {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut SystemNode<R> {
        &mut self.root
    }

    pub fn config(&self) -> &SystemConfig {
        &self.config.0
    }

    pub fn config_mut(&mut self) -> &mut SystemConfig {
        &mut self.config.0
    }
}

struct SystemConfigWrapper<R: Root>(SystemConfig, PhantomData<R>);

impl<R: Root> Serialize for SystemConfigWrapper<R> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        Ok(R::serialize_config(&self.0, serializer)?)
    }
}

impl<'de, R: Root> Deserialize<'de> for SystemConfigWrapper<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(SystemConfigWrapper(
            R::deserialize_config(deserializer)?,
            PhantomData,
        ))
    }
}

/// Handles Global Configuration of Systems
#[derive(Default)]
pub struct SystemConfig {
    configs: TypeIdMap<Box<dyn Config>>,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self {
            configs: TypeIdMap::default(),
        }
    }

    pub fn insert<T: Config>(&mut self, res: T) {
        self.configs.insert(TypeId::of::<T>(), Box::new(res));
    }

    pub fn remove<T: Config>(&mut self) {
        self.configs.remove(&TypeId::of::<T>());
    }

    pub fn get<T: Config>(&self) -> Option<&T> {
        self.configs
            .get(&TypeId::of::<T>())
            .map(|v| {
                let s: &dyn Any = v;
                s.downcast_ref::<T>()
            })
            .flatten()
    }

    pub fn get_mut<T: Config>(&mut self) -> Option<&mut T> {
        self.configs
            .get_mut(&TypeId::of::<T>())
            .map(|v| {
                let s: &mut dyn Any = v;
                s.downcast_mut::<T>()
            })
            .flatten()
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
