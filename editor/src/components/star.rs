use glam::Vec4;

pub struct Star {
    pub color: Vec4,
    pub shift: Vec4,
    pub radius: f32,

    pub granule_lacunarity: f32,
    pub granule_gain: f32,
    pub granule_octaves: f32,

    pub sunspot_sharpness: f32,
    pub sunspots_cutoff: f32,
    pub sunspots_frequency: f32,
}
