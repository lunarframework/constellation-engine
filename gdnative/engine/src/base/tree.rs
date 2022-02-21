use super::{Root, SerializableSystem, System, SystemNode};
use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::any::{Any, TypeId};
use std::fmt;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::marker::PhantomData;

pub struct SystemTree<R: System + Root> {
    root: SystemNode<R>,
    config: SystemConfigWrapper<R>,
}

impl<R: System + Root> SystemTree<R> {
    pub fn new(root: R) -> Self {
        let mut config = SystemConfig::new();
        R::init_config(&mut config);
        Self {
            root: SystemNode::new(root),
            config: SystemConfigWrapper(SystemConfig::new(), PhantomData),
        }
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

impl<'de, R: System + Root + SerializableSystem<'de>> Serialize for SystemTree<R> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut state = serializer.serialize_struct("SystemTree", 2)?;
        state.serialize_field("root", &self.root)?;
        state.serialize_field("config", &self.config)?;
        state.end()
    }
}

impl<'de, R: System + Root + SerializableSystem<'de>> Deserialize<'de> for SystemTree<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Root,
            Config,
        }

        struct SystemTreeVistor<'de, RPass: System + Root + SerializableSystem<'de>>(
            std::marker::PhantomData<&'de RPass>,
        );

        impl<'de, RPass: System + Root + SerializableSystem<'de>> Visitor<'de>
            for SystemTreeVistor<'de, RPass>
        {
            type Value = SystemTree<RPass>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SystemTree")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SystemTree<RPass>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let root = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let config = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SystemTree { root, config })
            }

            fn visit_map<V>(self, mut map: V) -> Result<SystemTree<RPass>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut root = None;
                let mut config = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Root => {
                            if root.is_some() {
                                return Err(de::Error::duplicate_field("root"));
                            }
                            root = Some(map.next_value()?);
                        }
                        Field::Config => {
                            if config.is_some() {
                                return Err(de::Error::duplicate_field("config"));
                            }
                            config = Some(map.next_value()?);
                        }
                    }
                }
                let root = root.ok_or_else(|| de::Error::missing_field("root"))?;
                let config = config.ok_or_else(|| de::Error::missing_field("config"))?;
                Ok(SystemTree { root, config })
            }
        }

        const FIELDS: &'static [&'static str] = &["root", "config"];
        deserializer.deserialize_struct(
            "SystemTree",
            FIELDS,
            SystemTreeVistor::<'de, R>(std::marker::PhantomData),
        )
    }
}

pub struct SystemConfigWrapper<R: Root>(SystemConfig, PhantomData<R>);

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
pub struct SystemConfig {
    configs: TypeIdMap<Box<dyn Any>>,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self {
            configs: TypeIdMap::default(),
        }
    }

    pub fn insert<T: Send + Sync + Any>(&mut self, res: T) {
        self.configs.insert(TypeId::of::<T>(), Box::new(res));
    }

    pub fn remove<T: Send + Sync + Any>(&mut self) {
        self.configs.remove(&TypeId::of::<T>());
    }

    pub fn get<T: Send + Sync + Any>(&self) -> Option<&T> {
        self.configs
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref::<T>())
            .flatten()
    }

    pub fn get_mut<T: Send + Sync + Any>(&mut self) -> Option<&mut T> {
        self.configs
            .get_mut(&TypeId::of::<T>())
            .map(|v| v.downcast_mut::<T>())
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
