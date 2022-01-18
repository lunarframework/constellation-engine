use glam::DVec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Star {
    temp: f32,
    pos: DVec3,
}
