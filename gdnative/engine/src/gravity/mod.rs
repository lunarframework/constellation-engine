use crate::base::{Root, System, SystemConfig};
// use crate::units::Units;
use hecs::World;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Serialize, Deserialize)]
pub struct GravitationalSystem;

impl System for GravitationalSystem {
    /// Sets up the system and adds any potential subsystems to the world.
    /// Recursively calls setup on any subsystems
    fn setup(
        &mut self,
        _children: &mut World,
        _config: &SystemConfig,
        _start_time: f64,
        _end_time: f64,
    ) {
    }

    /// Update the system and all subsystems
    fn update(&mut self, _children: &mut World, _config: &SystemConfig, _delta: f64) {}

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

    fn serialize_children<S>(_children: &World, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }

    fn deserialize_children<'de, D>(deserializer: D) -> Result<World, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UnitVisitor;

        impl<'de> Visitor<'de> for UnitVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A unit struct")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(())
            }
        }

        deserializer.deserialize_unit(UnitVisitor)?;

        Ok(World::new())
    }
}

impl Root for GravitationalSystem {
    fn default_config() -> SystemConfig {
        SystemConfig::new()
    }

    fn serialize_config<S>(_config: &SystemConfig, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit()
    }

    fn deserialize_config<'de, D>(deserializer: D) -> Result<SystemConfig, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UnitVisitor;

        impl<'de> Visitor<'de> for UnitVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A unit struct")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(())
            }
        }

        deserializer.deserialize_unit(UnitVisitor)?;

        Ok(SystemConfig::new())
    }
}
