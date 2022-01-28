use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::Star;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StarData {
    pub pos: DVec3,
    pub vel: DVec3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct View {
    pub max_time: f64,
    pub steps: u32,

    pub stars: Vec<Star>,
    pub data: Vec<StarData>,
}
