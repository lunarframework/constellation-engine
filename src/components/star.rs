use glam::Vec4;

pub struct Star {
    // Apearance
    pub granule_scale: f32,
    pub granule_lacunariy: f32,
    pub granule_freqency: f32,
    pub granule_octaves: f32,
    pub color: Vec4,

    pub sunspots_scale: f32,
    pub sunspots_offset: f32,
    pub sunspots_frequency: f32,
    pub sunspots_radius: f32,
}
