use super::SphereMesh;
use crate::universe::{Camera, Star, Transform};
use crate::RenderHandle;
use nalgebra::Matrix4;
use starlight::prelude::*;
use wgpu::BindGroup;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum StarResolution {
    /// Star is so far it can be rendered as particle
    Galaxy,
    /// Star is close enough to be rendered with some 3 Dimensionality
    Cluster,
    /// Star is in the local system and should be a full on sphere
    System5,
    System4,
    System3,
    System2,
    System1,
    System0,
    Orbit,
    Surface,
}

impl StarResolution {
    // pub fn to_resolution(self) -> u32 {
    //     match self {
    //         StarResolution::Galaxy => 2,
    //         StarResolution::Cluster => 2,
    //         StarResolution::System5 => 10,
    //         StarResolution::System4 => 20,
    //         StarResolution::System3 => 30,
    //         StarResolution::System2 => 40,
    //         StarResolution::System1 => 50,
    //         StarResolution::System0 => 60,
    //         StarResolution::Orbit => 70,
    //         StarResolution::Surface => 100,
    //     }
    // }

    pub fn to_index(self) -> usize {
        match self {
            StarResolution::Galaxy => 0,
            StarResolution::Cluster => 1,
            StarResolution::System5 => 2,
            StarResolution::System4 => 3,
            StarResolution::System3 => 4,
            StarResolution::System2 => 5,
            StarResolution::System1 => 6,
            StarResolution::System0 => 7,
            StarResolution::Orbit => 8,
            StarResolution::Surface => 9,
        }
    }
}

pub struct StarRenderer {
    renderer: RenderHandle,
    resolutions: Vec<SphereMesh>,
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,

    capacity: usize,
    count: usize,
    bind_groups: Vec<BindGroup>,
    staging_data: Vec<UniformBuffer>,
    uniforms: Option<wgpu::Buffer>,
}

impl<'b> StarRenderer {
    pub fn new(renderer: RenderHandle, output_format: wgpu::TextureFormat) -> Self {
        let mut resolutions = Vec::with_capacity(10);
        for i in 0..10 {
            resolutions.push(SphereMesh::new(i * 10 + 2));
        }

        let module = renderer
            .device()
            .create_shader_module(&wgpu::include_wgsl!("star.wgsl"));

        let uniform_bind_group_layout =
            renderer
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
            renderer
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("star_pipeline_layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            renderer
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

        // let size = std::mem::size_of::<UniformBuffer>();

        // let uniforms = renderer.device().create_buffer(&wgpu::BufferDescriptor {
        //     label: None,
        //     size: (Self::STAR_CHUNK * size) as u64,
        //     usage: wgpu::BufferUsages::UNIFORM,
        //     mapped_at_creation: false,
        // });

        // let mut bind_groups = Vec::with_capacity(Self::STAR_CHUNK);

        // for i in 0..Self::STAR_CHUNK {
        //     let offset = i * size;
        //     bind_groups.push(
        //         renderer
        //             .device()
        //             .create_bind_group(&wgpu::BindGroupDescriptor {
        //                 label: None,
        //                 entries: &[wgpu::BindGroupEntry {
        //                     binding: 0,
        //                     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                         buffer: &uniforms,
        //                         offset: offset as u64,
        //                         size: Some(std::num::NonZeroU64::new(size as u64).unwrap()),
        //                     }),
        //                 }],
        //                 layout: &uniform_bind_group_layout,
        //             }),
        //     );
        // }

        Self {
            renderer,
            resolutions,
            render_pipeline,
            bind_group_layout: uniform_bind_group_layout,
            capacity: 0,
            count: 0,
            staging_data: Vec::default(),
            uniforms: None,
            bind_groups: Vec::default(),
        }
    }

    pub fn prepare(&mut self, world: &World, camera: &Camera) {
        let mut query = world.query::<(&Transform, &Star)>();

        let count = query.iter().count();

        if self.capacity < count || self.uniforms.is_none() {
            self.capacity = count;
            self.uniforms = Some(
                self.renderer
                    .device()
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size: (count * std::mem::size_of::<UniformBuffer>()) as u64,
                        usage: wgpu::BufferUsages::UNIFORM,
                        mapped_at_creation: false,
                    }),
            );

            self.bind_groups.clear();
            self.bind_groups.reserve(count);

            self.staging_data.clear();
            self.staging_data.reserve(count);

            for i in 0..count {
                self.bind_groups.push(
                    self.renderer
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

        let proj_view_mat = camera.proj_view_mat();

        for (i, (_entity, (transform, _star))) in query.iter().enumerate() {
            self.staging_data[i].proj_view_model = proj_view_mat * transform.transform_mat();
        }

        if self.count > 0 {
            self.renderer.queue().write_buffer(
                (&self.uniforms).as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.staging_data[0..self.count]),
            );
        }
    }

    // pub fn render<'a>(&'b mut self, render_pass: &'a mut wgpu::RenderPass<'_>)
    // where
    //     'b: 'a,
    // {
    //     render_pass.set_pipeline(&self.render_pipeline);
    //     // render_pass.set_viewport(0.0, 0.0, width, height, 0.0, 1.0);
    // }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct UniformBuffer {
    proj_view_model: Matrix4<f32>,
}

use bytemuck::{Pod, Zeroable};

unsafe impl Pod for UniformBuffer {}

unsafe impl Zeroable for UniformBuffer {}
