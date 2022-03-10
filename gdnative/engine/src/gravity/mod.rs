use crate::base::{Root, System, SystemConfig, SystemNode};
use crate::global::Units;
use hecs::{serialize::column::*, Archetype, ColumnBatchBuilder, ColumnBatchType, World};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::{self, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::any::TypeId;

pub mod nbody;

#[derive(Serialize, Deserialize)]
pub struct GravitationalSystem;

impl System for GravitationalSystem {
    fn solve_begin(&mut self, children: &mut World, config: &SystemConfig, time: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.solve_begin(config, time);
        }
    }

    /// Update the system and all subsystems
    fn solve_update(&mut self, children: &mut World, config: &SystemConfig, time: f64, delta: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.solve_update(config, time, delta);
        }
    }

    fn solve_end(&mut self, children: &mut World, config: &SystemConfig, time: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.solve_end(config, time);
        }
    }

    fn view_begin(&mut self, children: &mut World, config: &SystemConfig, time: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.view_begin(config, time);
        }
    }

    fn view_set_time(&mut self, children: &mut World, config: &SystemConfig, time: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.view_set_time(config, time);
        }
    }

    fn view_end(&mut self, children: &mut World, config: &SystemConfig, time: f64) {
        for (_entity, nbody) in children.query_mut::<&mut SystemNode<nbody::NBodySystem>>() {
            nbody.view_end(config, time);
        }
    }

    fn serialize_system<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serialize(serializer)
    }

    fn deserialize_system<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize(deserializer)
    }

    fn serialize_children<S>(children: &World, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(children, &mut SeContext, serializer)
    }

    fn deserialize_children<'de, D>(deserializer: D) -> Result<World, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(&mut DeContext::default(), deserializer)
    }
}

impl Root for GravitationalSystem {
    fn default_config() -> SystemConfig {
        let mut config = SystemConfig::new();

        config.insert(Units::default());

        config
    }

    fn serialize_config<S>(config: &SystemConfig, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::Error;

        let mut seq = serializer.serialize_seq(Some(0))?;
        // seq.serialize_element(
        //     config
        //         .get::<Units>()
        //         .ok_or_else(|| S::Error::custom("config does not contain units"))?,
        // )?;
        seq.end()
    }

    fn deserialize_config<'de, D>(deserializer: D) -> Result<SystemConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigDeserializer;

        impl<'de> Visitor<'de> for ConfigDeserializer {
            type Value = SystemConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("SystemConfig sequence.")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                use de::Error;

                let mut config = SystemConfig::new();
                // config.insert::<Units>(
                //     seq.next_element()?
                //         .ok_or_else(|| A::Error::custom("config does not contain units"))?,
                // );
                Ok(config)
            }
        }

        deserializer.deserialize_seq(ConfigDeserializer)
    }
}

#[derive(Serialize, Deserialize)]
enum ComponentId {
    NBody,
}

struct SeContext;

impl SerializeContext for SeContext {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype
            .component_types()
            .filter(|&t| t == TypeId::of::<SystemNode<nbody::NBodySystem>>())
            .count()
    }

    fn serialize_component_ids<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: &mut S,
    ) -> Result<(), S::Error> {
        try_serialize_id::<SystemNode<nbody::NBodySystem>, _, _>(
            archetype,
            &ComponentId::NBody,
            out,
        )?;
        Ok(())
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: &mut S,
    ) -> Result<(), S::Error> {
        try_serialize::<SystemNode<nbody::NBodySystem>, _>(archetype, out)?;
        Ok(())
    }
}

// Could include references to external state for use by serialization methods
#[derive(Default)]
struct DeContext {
    /// Components of the archetype currently being deserialized
    components: Vec<ComponentId>,
}

impl DeserializeContext for DeContext {
    fn deserialize_component_ids<'de, A>(&mut self, mut seq: A) -> Result<ColumnBatchType, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        self.components.clear(); // Discard data from the previous archetype
        let mut batch = ColumnBatchType::new();
        while let Some(id) = seq.next_element()? {
            match id {
                ComponentId::NBody => {
                    batch.add::<SystemNode<nbody::NBodySystem>>();
                }
            }
            self.components.push(id);
        }
        Ok(batch)
    }

    fn deserialize_components<'de, A>(
        &mut self,
        entity_count: u32,
        mut seq: A,
        batch: &mut ColumnBatchBuilder,
    ) -> Result<(), A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Decode component data in the order that the component IDs appeared
        for component in &self.components {
            match *component {
                ComponentId::NBody => {
                    deserialize_column::<SystemNode<nbody::NBodySystem>, _>(
                        entity_count,
                        &mut seq,
                        batch,
                    )?;
                }
            }
        }
        Ok(())
    }
}
