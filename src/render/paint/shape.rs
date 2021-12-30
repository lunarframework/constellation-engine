#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color32(pub(crate) [u8; 4]);

impl Color32 {
    #[inline(always)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    #[inline(always)]
    pub const fn from_rgb_additive(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 0])
    }

    #[inline(always)]
    pub const fn from_rgba_premultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    #[inline(always)]
    pub fn r(&self) -> u8 {
        self.0[0]
    }

    #[inline(always)]
    pub fn g(&self) -> u8 {
        self.0[1]
    }

    #[inline(always)]
    pub fn b(&self) -> u8 {
        self.0[2]
    }

    #[inline(always)]
    pub fn a(&self) -> u8 {
        self.0[3]
    }

    #[inline(always)]
    pub fn to_tuple(&self) -> (u8, u8, u8, u8) {
        (self.r(), self.g(), self.b(), self.a())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: Color32,
}

pub struct Shape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
    pub image_id: ImageId,
}

impl Shape {
    pub fn with_image(image_id: ImageId) -> Self {
        Self {
            image_id,
            ..Default::default()
        }
    }

    /// Restore to default state, but without freeing memory.
    pub fn clear(&mut self) {
        self.indices.clear();
        self.vertices.clear();
        self.vertices = Default::default();
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ImageId(u64);
