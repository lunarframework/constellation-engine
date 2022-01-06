use super::Camera;
use super::RenderCtxRef;
use super::UniverseRenderer;
use crate::components::{Star, Transform};
use crate::render::CubeSphere;
use starlight::World;
use wgpu::util::DeviceExt;

pub struct StarPipeline {
    render: RenderCtxRef,
    // HD Rendering
    pipeline_hd: wgpu::RenderPipeline,
    env_bind_group_hd: wgpu::BindGroup,
    env_buffer_hd: wgpu::Buffer,
    env_data_hd: EnvBufferHd,
    star_bind_group_layout_hd: wgpu::BindGroupLayout,
    star_bind_groups_hd: Vec<wgpu::BindGroup>,
    star_data_hd: Vec<StarBufferHd>,
    star_buffer_hd: wgpu::Buffer,
    // pipeline: wgpu::RenderPipeline,

    // env_bind_group: wgpu::BindGroup,
    // env_buffer: wgpu::Buffer,
    // env_data: EnvBuffer,

    // index_count: u32,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,

    // instance_buffer_size: u64,
    // instance_buffer: wgpu::Buffer,
    // instance_data: Vec<InstanceBuffer>,
}

impl StarPipeline {
    pub fn new(render: RenderCtxRef) -> Self {
        // let module = render
        //     .device()
        //     .create_shader_module(&wgpu::include_wgsl!("shaders/star.wgsl"));

        // let env_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Star Env Buffer"),
        //     size: std::mem::size_of::<EnvBuffer>() as u64,
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        //     mapped_at_creation: false,
        // });

        // let env_bind_group_layout =
        //     render
        //         .device()
        //         .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //             label: Some("Star Env Bind Group Layout"),
        //             entries: &[wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::VERTEX,
        //                 ty: wgpu::BindingType::Buffer {
        //                     has_dynamic_offset: false,
        //                     min_binding_size: None,
        //                     ty: wgpu::BufferBindingType::Uniform,
        //                 },
        //                 count: None,
        //             }],
        //         });

        // let env_bind_group = render
        //     .device()
        //     .create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some("Star Env Bind Group"),
        //         layout: &env_bind_group_layout,
        //         entries: &[wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                 buffer: &env_buffer,
        //                 offset: 0,
        //                 size: None,
        //             }),
        //         }],
        //     });

        // let pipeline_layout =
        //     render
        //         .device()
        //         .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //             label: Some("star_pipeline_layout"),
        //             bind_group_layouts: &[&env_bind_group_layout],
        //             push_constant_ranges: &[],
        //         });

        // let pipeline = render
        //     .device()
        //     .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        //         label: Some("star_pipeline"),
        //         layout: Some(&pipeline_layout),
        //         vertex: wgpu::VertexState {
        //             entry_point: "vs_main",
        //             module: &module,
        //             buffers: &[
        //                 wgpu::VertexBufferLayout {
        //                     array_stride: 4 * 3,
        //                     step_mode: wgpu::VertexStepMode::Vertex,
        //                     // 0: vec3 position
        //                     attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        //                 },
        //                 wgpu::VertexBufferLayout {
        //                     array_stride: 4 * 4 * 6,
        //                     step_mode: wgpu::VertexStepMode::Instance,
        //                     // 1-4: mat4x4 transform
        //                     // 5: color
        //                     // 6: granule settings
        //                     attributes: &wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4]
        //                 },
        //             ],
        //         },
        //         primitive: wgpu::PrimitiveState {
        //             topology: wgpu::PrimitiveTopology::TriangleList,
        //             clamp_depth: false,
        //             conservative: false,
        //             cull_mode: Some(wgpu::Face::Back),
        //             front_face: wgpu::FrontFace::Cw,
        //             polygon_mode: wgpu::PolygonMode::Fill,
        //             strip_index_format: None,
        //         },
        //         depth_stencil: Some(wgpu::DepthStencilState {
        //             format: depth_format,
        //             depth_write_enabled: true,
        //             depth_compare: wgpu::CompareFunction::Less, // 1.
        //             stencil: wgpu::StencilState::default(),     // 2.
        //             bias: wgpu::DepthBiasState::default(),
        //         }),
        //         multisample: wgpu::MultisampleState {
        //             alpha_to_coverage_enabled: false,
        //             count: 1,
        //             mask: !0,
        //         },
        //         fragment: Some(wgpu::FragmentState {
        //             module: &module,
        //             entry_point: "fs_main",
        //             targets: &[wgpu::ColorTargetState {
        //                 format: output_format,
        //                 blend: Some(wgpu::BlendState {
        //                     color: wgpu::BlendComponent {
        //                         src_factor: wgpu::BlendFactor::One,
        //                         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        //                         operation: wgpu::BlendOperation::Add,
        //                     },
        //                     alpha: wgpu::BlendComponent {
        //                         src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
        //                         dst_factor: wgpu::BlendFactor::One,
        //                         operation: wgpu::BlendOperation::Add,
        //                     },
        //                 }),
        //                 write_mask: wgpu::ColorWrites::ALL,
        //             }],
        //         }),
        //     });

        let module_hd = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/star_hd.wgsl"));

        let env_buffer_hd = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star Env Buffer"),
            size: std::mem::size_of::<EnvBufferHd>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let env_bind_group_layout_hd =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Star Env Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Uniform,
                        },
                        count: None,
                    }],
                });

        let env_bind_group_hd = render
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Star Env Bind Group"),
                layout: &env_bind_group_layout_hd,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &env_buffer_hd,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let star_buffer_hd = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star HD Data Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let star_bind_group_layout_hd =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Star Env Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Uniform,
                        },
                        count: None,
                    }],
                });

        let pipeline_hd_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("star_hd_pipeline_layout"),
                    bind_group_layouts: &[&env_bind_group_layout_hd, &star_bind_group_layout_hd],
                    push_constant_ranges: &[],
                });

        let pipeline_hd = render
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("star_hd_pipeline"),
                layout: Some(&pipeline_hd_layout),
                vertex: wgpu::VertexState {
                    entry_point: "vs_main",
                    module: &module_hd,
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    clamp_depth: false,
                    conservative: false,
                    cull_mode: None,
                    front_face: wgpu::FrontFace::Cw,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    strip_index_format: None,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: render.depth_format(),
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module_hd,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: render.hdr_format(),
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

        // let mesh = CubeSphere::new(10);

        // let index_count = mesh.indices().len() as u32;

        // let vertex_buffer = render
        //     .device()
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: None,
        //         contents: bytemuck::cast_slice(mesh.vertices()),
        //         usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //     });

        // let index_buffer = render
        //     .device()
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: None,
        //         contents: bytemuck::cast_slice(mesh.indices()),
        //         usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        //     });

        // let instance_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Star Env Buffer"),
        //     size: 0,
        //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        //     mapped_at_creation: false,
        // });

        Self {
            render: render,
            pipeline_hd,
            env_bind_group_hd,
            env_buffer_hd,
            env_data_hd: EnvBufferHd::default(),
            star_buffer_hd,
            star_bind_group_layout_hd,
            star_bind_groups_hd: Vec::new(),
            star_data_hd: Vec::new(),
            // pipeline,

            // env_bind_group,
            // env_buffer,
            // env_data: EnvBuffer::default(),

            // index_count,
            // vertex_buffer,
            // index_buffer,

            // instance_buffer_size: 0,
            // instance_buffer,
            // instance_data: Vec::new(),
        }
    }

    pub fn update(&mut self, world: &World, camera: &Camera, dt: f32) {
        // **********************
        // Update Enviornment ***
        // **********************

        let proj_view = camera.compute_proj_view_matrix();
        self.env_data_hd.clip_to_world = proj_view.inverse();
        self.env_data_hd.world_to_clip = proj_view;
        self.env_data_hd.camera = camera.position().extend(camera.near());
        self.env_data_hd.time = 0.0;

        self.render.queue().write_buffer(
            &self.env_buffer_hd,
            0,
            bytemuck::cast_slice(&[self.env_data_hd]),
        );

        // *****************
        // Star ************
        // *****************

        let mut query = world.query::<(&Transform, &Star)>();

        let count = query.iter().count();

        self.star_data_hd.clear();
        self.star_bind_groups_hd.clear();
        self.star_data_hd.reserve(count);
        self.star_bind_groups_hd.reserve(count);

        for (_entity, (transform, star)) in query.iter() {
            self.star_data_hd.push(StarBufferHd {
                pos: transform.translation.extend(star.radius),
                color: star.color,
                shift: star.shift,
                granule_lacunarity: star.granule_lacunarity,
                granule_gain: star.granule_gain,
                granule_octaves: star.granule_octaves,
                sunspot_sharpness: star.sunspot_sharpness,
                sunspot_cutoff: star.sunspots_cutoff,
                sunspot_frequency: star.sunspots_frequency,
            });
        }

        if count > self.star_bind_groups_hd.len() {
            use std::alloc::Layout;
            use std::num::NonZeroU64;

            let layout = Layout::new::<StarBufferHd>().pad_to_align();

            self.star_buffer_hd = self.render.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Star HD Data Buffer"),
                size: (layout.size() * count) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

            self.star_bind_groups_hd.clear();
            self.star_bind_groups_hd.reserve(count);

            for i in 0..count {
                self.star_bind_groups_hd
                    .push(
                        self.render
                            .device()
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Star Bind Group"),
                                layout: &self.star_bind_group_layout_hd,
                                entries: &[wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer: &self.star_buffer_hd,
                                        offset: (layout.size() * i) as u64,
                                        size: Some(NonZeroU64::new(layout.size() as u64).unwrap()),
                                    }),
                                }],
                            }),
                    );
            }
        }

        self.render.queue().write_buffer(
            &self.star_buffer_hd,
            0,
            bytemuck::cast_slice(self.star_data_hd.as_slice()),
        );

        // let proj_view = camera.compute_proj_view_matrix();
        // self.env_data.proj_view = proj_view;
        // self.env_data.camera_pos = camera.position().extend(1.0);
        // self.env_data.anim_time = 0.0;

        // self.render.queue().write_buffer(
        //     &self.env_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.env_data]),
        // );

        // // ***********************
        // // Update Instances ******
        // // ***********************

        // let mut query = world.query::<(&Transform, &Star)>();

        // let count = query.iter().count();

        // self.instance_data.clear();
        // self.instance_data.reserve(count);

        // for (_entity, (transform, star)) in query.iter() {
        //     self.instance_data.push(InstanceBuffer {
        //         transform: transform.compute_matrix(),
        //         color: star.color,
        //         shifted_color: star.shifted_color,
        //         ganules: Vec4::new(
        //             star.granule_scale,
        //             star.granule_lacunariy,
        //             star.granule_freqency,
        //             star.granule_octaves,
        //         ),
        //         sunspots: Vec4::new(
        //             star.sunspots_scale,
        //             star.sunspots_offset,
        //             star.sunspots_frequency,
        //             star.sunspots_radius,
        //         ),
        //     });
        // }

        // if (count * std::mem::size_of::<InstanceBuffer>()) as u64 > self.instance_buffer_size {
        //     self.instance_buffer_size = (count * std::mem::size_of::<InstanceBuffer>()) as u64;
        //     self.instance_buffer = self.render.device().create_buffer(&wgpu::BufferDescriptor {
        //         label: Some("Star Env Buffer"),
        //         size: self.instance_buffer_size,
        //         usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        //         mapped_at_creation: false,
        //     });
        // }

        // self.render.queue().write_buffer(
        //     &self.instance_buffer,
        //     0,
        //     bytemuck::cast_slice(self.instance_data.as_slice()),
        // );
    }

    pub fn render<'s, 'r>(&'s self, render_pass: &'r mut wgpu::RenderPass<'s>, camera: &Camera) {
        // HD Rendering
        render_pass.set_pipeline(&self.pipeline_hd);
        render_pass.set_viewport(
            0.0,
            0.0,
            camera.width() as f32,
            camera.height() as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(0, 0, camera.width(), camera.height());
        render_pass.set_bind_group(0, &self.env_bind_group_hd, &[]);

        for bind_group in self.star_bind_groups_hd.iter() {
            render_pass.set_bind_group(1, bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        // render_pass.set_pipeline(&self.pipeline);
        // render_pass.set_viewport(
        //     0.0,
        //     0.0,
        //     camera.width() as f32,
        //     camera.height() as f32,
        //     0.0,
        //     1.0,
        // );
        // render_pass.set_scissor_rect(0, 0, camera.width(), camera.height());
        // render_pass.set_bind_group(0, &self.env_bind_group, &[]);
        // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        // render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_data.len() as u32);
    }
}

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct EnvBufferHd {
    clip_to_world: Mat4,
    world_to_clip: Mat4,
    /// x, y, z = position
    /// w = near plane
    camera: Vec4,
    time: f32,
}

unsafe impl Pod for EnvBufferHd {}

unsafe impl Zeroable for EnvBufferHd {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct StarBufferHd {
    pos: Vec4,
    color: Vec4,
    shift: Vec4,
    granule_lacunarity: f32,
    granule_gain: f32,
    granule_octaves: f32,
    sunspot_sharpness: f32,
    sunspot_cutoff: f32,
    sunspot_frequency: f32,
}

unsafe impl Pod for StarBufferHd {}

unsafe impl Zeroable for StarBufferHd {}

// #[repr(C)]
// #[derive(Copy, Clone, Default)]
// struct EnvBuffer {
//     proj_view: Mat4,
//     camera_pos: Vec4,
//     anim_time: f32,
// }

// unsafe impl Pod for EnvBuffer {}

// unsafe impl Zeroable for EnvBuffer {}

// #[repr(C)]
// #[derive(Copy, Clone, Default)]
// struct InstanceBuffer {
//     transform: Mat4,
//     color: Vec4,
//     shifted_color: Vec4,
//     ganules: Vec4,
//     sunspots: Vec4,
// }

// unsafe impl Pod for InstanceBuffer {}

// unsafe impl Zeroable for InstanceBuffer {}

// // Needed since we can't use bytemuck for external types.
// fn as_byte_slice<T>(slice: &[T]) -> &[u8] {
//     let len = slice.len() * std::mem::size_of::<T>();
//     let ptr = slice.as_ptr() as *const u8;
//     unsafe { std::slice::from_raw_parts(ptr, len) }
// }
