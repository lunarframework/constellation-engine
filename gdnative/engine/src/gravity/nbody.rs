use crate::base::{AbstractVector, ContinuousRecord, System, SystemConfig};
use crate::global::Units;
use gdnative::core_types::Rid;
use glam::DVec3;
use hecs::{serialize::column::*, Archetype, ColumnBatchBuilder, ColumnBatchType, World};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::TypeId;

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub pos: DVec3,
}

impl AbstractVector for Position {
    fn zero() -> Self {
        Self { pos: DVec3::ZERO }
    }

    fn one() -> Self {
        Self { pos: DVec3::ONE }
    }

    fn add(&mut self, other: Self) {
        self.pos += other.pos;
    }
    fn scale(&mut self, scalar: f64) {
        self.pos *= scalar;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NBody {
    pub index: usize,
    pub pos: DVec3,
    pub vel: DVec3,
    pub mass: f64,
}

#[derive(Serialize, Deserialize)]
enum ComponentId {
    Body,
    Record,
}

struct SeContext;

impl SerializeContext for SeContext {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype
            .component_types()
            .filter(|&t| {
                t == TypeId::of::<NBody>() || t == TypeId::of::<ContinuousRecord<Position>>()
            })
            .count()
    }

    fn serialize_component_ids<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: &mut S,
    ) -> Result<(), S::Error> {
        try_serialize_id::<NBody, _, _>(archetype, &ComponentId::Body, out)?;
        try_serialize_id::<ContinuousRecord<Position>, _, _>(archetype, &ComponentId::Record, out)?;
        Ok(())
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: &mut S,
    ) -> Result<(), S::Error> {
        try_serialize::<NBody, _>(archetype, out)?;
        try_serialize::<ContinuousRecord<Position>, _>(archetype, out)?;
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
                ComponentId::Body => {
                    batch.add::<NBody>();
                }
                ComponentId::Record => {
                    batch.add::<ContinuousRecord<Position>>();
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
                ComponentId::Body => {
                    deserialize_column::<NBody, _>(entity_count, &mut seq, batch)?;
                }
                ComponentId::Record => {
                    deserialize_column::<ContinuousRecord<Position>, _>(
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

#[derive(Serialize, Deserialize)]
pub struct NBodySystem;

impl System for NBodySystem {
    fn solve_begin(&mut self, _children: &mut World, _config: &SystemConfig, _time: f64) {}

    /// Update the system and all subsystems
    fn solve_update(
        &mut self,
        children: &mut World,
        _config: &SystemConfig,
        time: f64,
        delta: f64,
    ) {
        // let units = config.get::<Units>().unwrap();

        for (_entity, (star, record)) in
            children.query_mut::<(&NBody, &mut ContinuousRecord<Position>)>()
        {
            record.save(
                time,
                Position {
                    pos: star.pos.clone(),
                },
            )
        }

        // let c = units.speed_of_light();
        // let g = units.gravitational_constant();
        let c = 1.0;
        let g = 1.0;
        let c_sq = c * c;

        let nbodystars = children
            .query_mut::<&NBody>()
            .into_iter()
            .map(|(_e, star)| star.clone())
            .collect::<Vec<_>>();

        for (_entity, orbiting) in children.query_mut::<&mut NBody>() {
            let mut acc = DVec3::new(0.0, 0.0, 0.0);

            for grav in nbodystars.iter() {
                let rel_pos = orbiting.pos - grav.pos;
                let r = rel_pos.length();

                if r < 1.0e-10 {
                    continue;
                }

                let rel_vel = orbiting.vel - grav.vel;
                let rel_vel_sq = DVec3::new(
                    rel_vel.x * rel_vel.x,
                    rel_vel.y * rel_vel.y,
                    rel_vel.z * rel_vel.z,
                );
                let rel_vel_pos = DVec3::new(
                    rel_pos.x * rel_vel.x,
                    rel_pos.y * rel_vel.y,
                    rel_pos.z * rel_vel.z,
                );

                let mu = g * grav.mass;

                let force_over_r = -mu / (r * r * r);
                acc.x += force_over_r * rel_pos.x;
                acc.y += force_over_r * rel_pos.y;
                acc.z += force_over_r * rel_pos.z;

                // let m = mu / (2.0 * c_sq * r);
                // let one_over_c_sq_one_plus_m = 1.0 / (c_sq * (1.0 + m));
                // let one_minus_m_over_one_plus_m = (1.0 - m)
                //     / ((1.0 + m)
                //         * (1.0 + m)
                //         * (1.0 + m)
                //         * (1.0 + m)
                //         * (1.0 + m)
                //         * (1.0 + m)
                //         * (1.0 + m));
                // let rel_pos_dot_vel_over_one_minus_m =
                //     (rel_vel_pos.x + rel_vel_pos.y + rel_vel_pos.z) / (1.0 - m);

                // acc.x += force_over_r
                //     * (one_minus_m_over_one_plus_m * rel_pos.x
                //         - one_over_c_sq_one_plus_m
                //             * (rel_pos.x * (rel_vel_sq.x - rel_vel_sq.y - rel_vel_sq.z)
                //                 + 2.0
                //                     * rel_vel.x
                //                     * (rel_vel_pos.y
                //                         + rel_vel_pos.z
                //                         + rel_pos_dot_vel_over_one_minus_m)));

                // acc.y += force_over_r
                //     * (one_minus_m_over_one_plus_m * rel_pos.y
                //         - one_over_c_sq_one_plus_m
                //             * (rel_pos.y * (rel_vel_sq.y - rel_vel_sq.x - rel_vel_sq.z)
                //                 + 2.0
                //                     * rel_vel.y
                //                     * (rel_vel_pos.x
                //                         + rel_vel_pos.z
                //                         + rel_pos_dot_vel_over_one_minus_m)));

                // acc.z += force_over_r
                //     * (one_minus_m_over_one_plus_m * rel_pos.z
                //         - one_over_c_sq_one_plus_m
                //             * (rel_pos.z * (rel_vel_sq.z - rel_vel_sq.y - rel_vel_sq.x)
                //                 + 2.0
                //                     * rel_vel.z
                //                     * (rel_vel_pos.y
                //                         + rel_vel_pos.x
                //                         + rel_pos_dot_vel_over_one_minus_m)));
            }

            orbiting.vel += acc * delta;
            orbiting.pos += orbiting.vel * delta;
        }
    }

    fn solve_end(&mut self, children: &mut World, _config: &SystemConfig, time: f64) {
        for (_entity, (star, record)) in
            children.query_mut::<(&NBody, &mut ContinuousRecord<Position>)>()
        {
            record.save(
                time,
                Position {
                    pos: star.pos.clone(),
                },
            )
        }
    }

    fn view_begin(&mut self, _children: &mut World, _config: &SystemConfig, _time: f64) {}

    fn view_set_time(&mut self, _children: &mut World, _config: &SystemConfig, _time: f64) {}

    fn view_end(&mut self, _children: &mut World, _config: &SystemConfig, _time: f64) {}

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
