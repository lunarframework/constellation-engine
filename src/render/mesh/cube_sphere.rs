use glam::{Vec2, Vec3};

pub struct CubeSphere {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
}

impl CubeSphere {
    pub fn new(resolution: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        append_face(Vec3::X, resolution, &mut vertices, &mut indices);
        append_face(-Vec3::X, resolution, &mut vertices, &mut indices);
        append_face(Vec3::Y, resolution, &mut vertices, &mut indices);
        append_face(-Vec3::Y, resolution, &mut vertices, &mut indices);
        append_face(Vec3::Z, resolution, &mut vertices, &mut indices);
        append_face(-Vec3::Z, resolution, &mut vertices, &mut indices);

        for vertex in vertices.iter_mut() {
            let x2 = vertex.x * vertex.x;
            let y2 = vertex.y * vertex.y;
            let z2 = vertex.z * vertex.z;

            vertex.x *= (1.0 - (y2 + z2) / 2.0 + (y2 * z2) / 3.0).sqrt();
            vertex.y *= (1.0 - (z2 + x2) / 2.0 + (z2 * x2) / 3.0).sqrt();
            vertex.z *= (1.0 - (x2 + y2) / 2.0 + (x2 * y2) / 3.0).sqrt();
        }

        Self { vertices, indices }
    }

    pub fn vertices(&self) -> &[Vec3] {
        self.vertices.as_slice()
    }

    pub fn indices(&self) -> &[u32] {
        self.indices.as_slice()
    }
}

fn append_face(normal: Vec3, resolution: u32, vertices: &mut Vec<Vec3>, indices: &mut Vec<u32>) {
    let axis_a = Vec3::new(normal.y, normal.z, normal.x);
    let axis_b = normal.cross(axis_a);

    let vertex_offset = vertices.len();

    vertices.reserve((resolution * resolution) as usize);
    indices.reserve(((resolution - 1) * (resolution - 1) * 6) as usize);

    for y in 0..resolution {
        for x in 0..resolution {
            let vertex_index = x + y * resolution + vertex_offset as u32;
            let t = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point = normal + axis_a * (2.0 * t.x - 1.0) + axis_b * (2.0 * t.y - 1.0);
            vertices.push(point);

            if x != (resolution - 1) && y != (resolution - 1) {
                indices.push(vertex_index);
                indices.push(vertex_index + resolution + 1);
                indices.push(vertex_index + resolution);
                indices.push(vertex_index);
                indices.push(vertex_index + 1);
                indices.push(vertex_index + resolution + 1);
            }
        }
    }
}
