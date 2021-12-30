use nalgebra::{Vector2, Vector3};

#[derive(Default, Clone, Copy)]
pub struct SphereVertex {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
}

#[derive(Default, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vector3<f32>>,
    pub triangles: Vec<u32>,
}

pub struct SphereMesh {
    pub faces: [MeshData; 6],
}

impl SphereMesh {
    pub fn new(resolution: u32) -> Self {
        let mut faces = create_faces(resolution);

        for face in faces.iter_mut() {
            for vertex in face.vertices.iter_mut() {
                let x2 = vertex.x * vertex.x;
                let y2 = vertex.y * vertex.y;
                let z2 = vertex.z * vertex.z;

                vertex.x *= (1.0 - (y2 * z2) / 2.0 + (y2 * z2) / 3.0).sqrt();
                vertex.y *= (1.0 - (z2 * x2) / 2.0 + (z2 * x2) / 3.0).sqrt();
                vertex.z *= (1.0 - (x2 * y2) / 2.0 + (x2 * y2) / 3.0).sqrt();
            }
        }

        Self { faces }
    }
}

fn create_face(normal: Vector3<f32>, resolution: u32) -> MeshData {
    assert!(resolution > 1, "Resolution must be larger than 1");
    let axis_a = Vector3::from([normal.x, normal.z, normal.y]);
    let axis_b = normal.cross(&axis_a);
    let mut vertices = vec![Vector3::<f32>::default(); resolution as usize];
    let mut triangles = vec![0u32; ((resolution - 1) * (resolution - 1) * 6) as usize];

    let mut tri_index = 0usize;

    for y in 0..resolution {
        for x in 0..resolution {
            let vertex_index = x + y * resolution;
            let t = Vector2::from([x as f32, y as f32]) / (resolution - 1) as f32;
            let point = normal + axis_a * (2.0 * t.x - 1.0) + axis_b * (2.0 * t.y - 1.0);
            vertices[vertex_index as usize] = point;

            if x != (resolution - 1) && y != (resolution - 1) {
                triangles[tri_index + 0] = vertex_index;
                triangles[tri_index + 1] = vertex_index + resolution + 1;
                triangles[tri_index + 2] = vertex_index + resolution;
                triangles[tri_index + 3] = vertex_index;
                triangles[tri_index + 0] = vertex_index + 1;
                triangles[tri_index + 1] = vertex_index + resolution + 1;
                tri_index += 6;
            }
        }
    }

    MeshData {
        vertices,
        triangles,
    }
}

// TODO Optimize this (and ideally fit all meshes into one)

fn create_faces(resolution: u32) -> [MeshData; 6] {
    let mut all_faces = [
        MeshData::default(),
        MeshData::default(),
        MeshData::default(),
        MeshData::default(),
        MeshData::default(),
        MeshData::default(),
    ];

    let face_normals: [Vector3<f32>; 6] = [
        Vector3::x(),
        -Vector3::x(),
        Vector3::y(),
        -Vector3::y(),
        Vector3::z(),
        -Vector3::z(),
    ];

    for (i, &normal) in face_normals.iter().enumerate() {
        all_faces[i] = create_face(normal, resolution);
    }

    all_faces
}
