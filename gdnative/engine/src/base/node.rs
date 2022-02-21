use super::{SerializableSystem, System};
use hecs::World;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

pub struct SystemNode<S: System> {
    system: S,
    children: Children<S>,
}

impl<S: System> SystemNode<S> {
    pub fn new(system: S) -> Self {
        let children = World::new();

        Self {
            system,
            children: Children::new(children),
        }
    }

    pub fn get(&self) -> &S {
        &self.system
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.system
    }

    pub fn children(&self) -> &World {
        &self.children.0
    }

    pub fn children_mut(&mut self) -> &mut World {
        &mut self.children.0
    }
}

impl<'de, S: System + SerializableSystem<'de>> Serialize for SystemNode<S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut state = serializer.serialize_struct("SystemNode", 2)?;
        state.serialize_field("system", &self.system)?;
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

impl<'de, S: System + SerializableSystem<'de>> Deserialize<'de> for SystemNode<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            System,
            Children,
        }

        struct SystemNodeVistor<'de, SPass: System + SerializableSystem<'de>>(
            std::marker::PhantomData<&'de SPass>,
        );

        impl<'de, SPass: System + SerializableSystem<'de>> Visitor<'de> for SystemNodeVistor<'de, SPass> {
            type Value = SystemNode<SPass>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SystemNode")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SystemNode<SPass>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let system = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let children = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SystemNode { system, children })
            }

            fn visit_map<V>(self, mut map: V) -> Result<SystemNode<SPass>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut system = None;
                let mut children = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::System => {
                            if system.is_some() {
                                return Err(de::Error::duplicate_field("system"));
                            }
                            system = Some(map.next_value()?);
                        }
                        Field::Children => {
                            if children.is_some() {
                                return Err(de::Error::duplicate_field("children"));
                            }
                            children = Some(map.next_value()?);
                        }
                    }
                }
                let system = system.ok_or_else(|| de::Error::missing_field("system"))?;
                let children = children.ok_or_else(|| de::Error::missing_field("children"))?;
                Ok(SystemNode { system, children })
            }
        }

        const FIELDS: &'static [&'static str] = &["system", "children"];
        deserializer.deserialize_struct(
            "SystemNode",
            FIELDS,
            SystemNodeVistor::<'de, S>(std::marker::PhantomData),
        )
    }
}

struct Children<S: System>(World, std::marker::PhantomData<S>);

impl<S: System> Children<S> {
    fn new(world: World) -> Self {
        Self(world, std::marker::PhantomData)
    }
}

impl<'de, S: System + SerializableSystem<'de>> Serialize for Children<S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        Ok(hecs::serialize::column::serialize(
            &self.0,
            &mut S::ser_context(),
            serializer,
        )?)
    }
}

impl<'de, S: System + SerializableSystem<'de>> Deserialize<'de> for Children<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(hecs::serialize::column::deserialize(
            &mut S::de_context(),
            deserializer,
        )?))
    }
}
