use super::Camera;
use super::RenderCtxRef;
use crate::components::{Star, Transform};
use starlight::World;
use wgpu::util::DeviceExt;

pub struct StarPipeline {
    render: RenderCtxRef,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,

    capacity: usize,
    count: usize,
    bind_groups: Vec<wgpu::BindGroup>,
    staging_data: Vec<UniformBuffer>,
    uniforms: Option<wgpu::Buffer>,

    indices_per_buffer: u32,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<wgpu::Buffer>,
}

impl StarPipeline {
    pub fn new(render: RenderCtxRef, output_format: wgpu::TextureFormat) -> Self {
        let module = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/star.wgsl"));

        let uniform_bind_group_layout =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("star_uniform_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Uniform,
                        },
                        count: None,
                    }],
                });

        let pipeline_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("star_pipeline_layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = render
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("star_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    entry_point: "vs_main",
                    module: &module,
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 3 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        // 0: vec3 position
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    clamp_depth: false,
                    conservative: false,
                    cull_mode: None,
                    front_face: wgpu::FrontFace::default(),
                    polygon_mode: wgpu::PolygonMode::default(),
                    strip_index_format: None,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: output_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
            });

        let mesh = StarMesh::new(10);

        let mut vertex_buffers = Vec::with_capacity(6);
        let mut index_buffers = Vec::with_capacity(6);

        let mut indices_per_buffer = 0;

        for face in mesh.faces.iter() {
            vertex_buffers.push(render.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: as_byte_slice(face.vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ));

            index_buffers.push(render.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: as_byte_slice(face.triangles.as_slice()),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                },
            ));

            indices_per_buffer = face.triangles.len() as u32;
        }

        Self {
            render: render,
            pipeline,
            bind_group_layout: uniform_bind_group_layout,
            capacity: 0,
            count: 0,
            staging_data: Vec::default(),
            uniforms: None,
            bind_groups: Vec::default(),

            indices_per_buffer,
            vertex_buffers,
            index_buffers,
        }
    }

    pub fn update(&mut self, world: &World, camera: &Camera) {
        let mut query = world.query::<(&Transform, &Star)>();

        let count = query.iter().count();

        if self.capacity < count || self.uniforms.is_none() {
            self.capacity = count;
            self.uniforms = Some(self.render.device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (count * std::mem::size_of::<UniformBuffer>()) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

            self.bind_groups.clear();
            self.bind_groups.reserve(count);

            self.staging_data.clear();
            self.staging_data.reserve(count);

            for i in 0..count {
                self.bind_groups.push(
                    self.render
                        .device()
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: (&self.uniforms).as_ref().unwrap(),
                                    offset: (i * std::mem::size_of::<UniformBuffer>()) as u64,
                                    size: Some(
                                        std::num::NonZeroU64::new(
                                            std::mem::size_of::<UniformBuffer>() as u64,
                                        )
                                        .unwrap(),
                                    ),
                                }),
                            }],
                        }),
                );

                self.staging_data.push(UniformBuffer::default());
            }
        }

        self.count = count;

        let proj_view = camera.compute_proj_view_matrix();

        for (i, (_entity, (transform, _star))) in query.iter().enumerate() {
            self.staging_data[i].proj_view_model = proj_view * transform.compute_matrix();
        }

        if self.count > 0 {
            self.render.queue().write_buffer(
                (&self.uniforms).as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.staging_data[0..self.count]),
            );
        }
    }

    pub fn render<'s, 'r>(&'s self, render_pass: &'r mut wgpu::RenderPass<'s>, camera: &Camera) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_viewport(
            0.0,
            0.0,
            camera.width() as f32,
            camera.height() as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(0, 0, camera.width(), camera.height());

        for bind_group in self.bind_groups.iter() {
            render_pass.set_bind_group(0, bind_group, &[]);

            for i in 0..6 {
                render_pass.set_vertex_buffer(0, self.vertex_buffers[i].slice(..));
                render_pass
                    .set_index_buffer(self.index_buffers[i].slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.indices_per_buffer, 0, 0..1);
            }
        }
    }
}

use glam::{Mat4, Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct UniformBuffer {
    proj_view_model: Mat4,
}

use bytemuck::{Pod, Zeroable};

unsafe impl Pod for UniformBuffer {}

unsafe impl Zeroable for UniformBuffer {}

// Needed since we can't use bytemuck for external types.
fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
    let len = slice.len() * std::mem::size_of::<T>();
    let ptr = slice.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

#[derive(Default, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<u32>,
}

pub struct StarMesh {
    pub faces: [MeshData; 6],
}

impl StarMesh {
    pub fn new(resolution: u32) -> Self {
        let mut faces = create_faces(resolution);

        for face in faces.iter_mut() {
            for vertex in face.vertices.iter_mut() {
                *vertex = vertex.normalize();

                // let x2 = vertex.x * vertex.x;
                // let y2 = vertex.y * vertex.y;
                // let z2 = vertex.z * vertex.z;

                // vertex.x *= (1.0 - (y2 + z2) / 2.0 + (y2 * z2) / 3.0).sqrt();
                // vertex.y *= (1.0 - (z2 + x2) / 2.0 + (z2 * x2) / 3.0).sqrt();
                // vertex.z *= (1.0 - (x2 + y2) / 2.0 + (x2 * y2) / 3.0).sqrt();
            }
        }

        Self { faces }
    }
}

fn create_face(normal: Vec3, resolution: u32) -> MeshData {
    assert!(resolution > 1, "Resolution must be larger than 1");
    let axis_a = Vec3::new(normal.x, normal.z, normal.y);
    let axis_b = normal.cross(axis_a);
    let mut vertices = vec![Vec3::zeroed(); (resolution * resolution) as usize];
    let mut triangles = vec![0u32; ((resolution - 1) * (resolution - 1) * 6) as usize];

    let mut tri_index = 0usize;

    for y in 0..resolution {
        for x in 0..resolution {
            let vertex_index = x + y * resolution;
            let t = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point = normal + axis_a * (2.0 * t.x - 1.0) + axis_b * (2.0 * t.y - 1.0);
            vertices[vertex_index as usize] = point;

            if x != (resolution - 1) && y != (resolution - 1) {
                triangles[tri_index + 0] = vertex_index;
                triangles[tri_index + 1] = vertex_index + resolution + 1;
                triangles[tri_index + 2] = vertex_index + resolution;
                triangles[tri_index + 3] = vertex_index;
                triangles[tri_index + 4] = vertex_index + 1;
                triangles[tri_index + 5] = vertex_index + resolution + 1;
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

    let face_normals: [Vec3; 6] = [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z];

    for (i, &normal) in face_normals.iter().enumerate() {
        all_faces[i] = create_face(normal, resolution);
    }

    all_faces
}
