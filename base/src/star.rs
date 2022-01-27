use glam::DVec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Star {
    pub temp: f32,
    pub pos: DVec3,
    pub vel: DVec3,
    pub mass: f64,
}
