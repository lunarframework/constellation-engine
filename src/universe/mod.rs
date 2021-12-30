pub mod camera;
pub mod components;
pub mod render;

pub use camera::Camera;
pub use components::{Star, Transform};
pub use render::{MeshData, SphereMesh, StarRenderer, StarResolution};
