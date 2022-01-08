use super::Camera;
use super::RenderCtxRef;
use super::UniverseRenderer;
use crate::components::{Star, Transform};
use crate::render::CubeSphere;
use starlight::World;
use std::alloc::Layout;
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

pub struct StarSettings {
    pub anim_time: f32,
    pub min_size_for_rays: f32,
}

pub struct StarPipeline {
    render: RenderCtxRef,

    pipeline_rays: wgpu::RenderPipeline,
    env_bind_group_rays: wgpu::BindGroup,
    env_buffer_rays: wgpu::Buffer,
    env_data_rays: EnvBufferRays,
    star_bind_group_layout_rays: wgpu::BindGroupLayout,
    star_bind_groups_rays: Vec<wgpu::BindGroup>,
    star_data_rays: Vec<StarBufferRays>,
    star_buffer_rays: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,
    env_bind_group: wgpu::BindGroup,
    env_buffer: wgpu::Buffer,
    env_data: EnvBuffer,
    star_instance_count: usize,
    star_instance_buffer: wgpu::Buffer,
    star_instance_data: Vec<StarInstanceBuffer>,
}

impl StarPipeline {
    pub fn new(render: RenderCtxRef) -> Self {
        let module = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/star.wgsl"));

        let env_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star Env Buffer"),
            size: std::mem::size_of::<EnvBuffer>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let env_bind_group_layout =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Star Env Bind Group Layout"),
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

        let env_bind_group = render
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Star Env Bind Group"),
                layout: &env_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: env_buffer.as_entire_binding(),
                }],
            });

        let pipeline_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("star_pipeline_layout"),
                    bind_group_layouts: &[&env_bind_group_layout],
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
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 4 * 4 * 3,
                            step_mode: wgpu::VertexStepMode::Instance,
                            // 0: pos,
                            // 1: color,
                            // 2: shift
                            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4]
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    clamp_depth: false,
                    conservative: false,
                    cull_mode: Some(wgpu::Face::Back),
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
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: render.hdr_format(),
                        // blend: Some(wgpu::BlendState {
                        //     color: wgpu::BlendComponent {
                        //         src_factor: wgpu::BlendFactor::One,
                        //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        //         operation: wgpu::BlendOperation::Add,
                        //     },
                        //     alpha: wgpu::BlendComponent {
                        //         src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                        //         dst_factor: wgpu::BlendFactor::One,
                        //         operation: wgpu::BlendOperation::Add,
                        //     },
                        // }),
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
            });

        let star_instance_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star Instance Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let module_rays = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/star_rays.wgsl"));

        let env_buffer_rays = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star Env Buffer"),
            size: std::mem::size_of::<EnvBufferRays>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let env_bind_group_layout_rays =
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

        let env_bind_group_rays = render
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Star Env Bind Group"),
                layout: &env_bind_group_layout_rays,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &env_buffer_rays,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let star_buffer_rays = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Star HD Data Buffer"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let star_bind_group_layout_rays =
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

        let pipeline_rays_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("star_rays_pipeline_layout"),
                    bind_group_layouts: &[
                        &env_bind_group_layout_rays,
                        &star_bind_group_layout_rays,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline_rays =
            render
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("star_rays_pipeline"),
                    layout: Some(&pipeline_rays_layout),
                    vertex: wgpu::VertexState {
                        entry_point: "vs_main",
                        module: &module_rays,
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
                        module: &module_rays,
                        entry_point: "fs_main",
                        targets: &[wgpu::ColorTargetState {
                            format: render.hdr_format(),
                            // blend: Some(wgpu::BlendState {
                            //     color: wgpu::BlendComponent {
                            //         src_factor: wgpu::BlendFactor::One,
                            //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            //         operation: wgpu::BlendOperation::Add,
                            //     },
                            //     alpha: wgpu::BlendComponent {
                            //         src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            //         dst_factor: wgpu::BlendFactor::One,
                            //         operation: wgpu::BlendOperation::Add,
                            //     },
                            // }),
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                });

        Self {
            render: render,
            pipeline_rays,
            env_bind_group_rays,
            env_buffer_rays,
            env_data_rays: EnvBufferRays::default(),
            star_buffer_rays,
            star_bind_group_layout_rays,
            star_bind_groups_rays: Vec::new(),
            star_data_rays: Vec::new(),

            pipeline,
            env_bind_group,
            env_buffer,
            env_data: EnvBuffer::default(),
            star_instance_count: 0,
            star_instance_buffer,
            star_instance_data: Vec::new(),
        }
    }

    pub fn prepare(&mut self, world: &World, camera: &Camera, settings: &StarSettings) {
        // **********************
        // Update Enviornment ***
        // **********************

        let proj_view = camera.compute_proj_view_matrix();
        let proj = camera.compute_projection_matrix();
        let view = camera.compute_view_matrix();

        self.env_data_rays.inv_proj_view = proj_view.inverse();
        self.env_data_rays.proj_view = proj_view;
        self.env_data_rays.camera = camera.position().extend(1.0);
        self.env_data_rays.near = camera.near();
        self.env_data_rays.far = camera.far();
        self.env_data_rays.time = settings.anim_time;

        self.render.queue().write_buffer(
            &self.env_buffer_rays,
            0,
            bytemuck::cast_slice(&[self.env_data_rays]),
        );

        self.env_data.proj = proj;
        self.env_data.view = view;

        self.render.queue().write_buffer(
            &self.env_buffer,
            0,
            bytemuck::cast_slice(&[self.env_data]),
        );

        // *****************
        // Stars ***********
        // *****************

        self.star_data_rays.clear();
        self.star_instance_data.clear();

        let mut query = world.query::<(&Transform, &Star)>();

        for (_entity, (transform, star)) in query.iter() {
            let distance = transform.translation.distance(*camera.position()) - star.radius;
            let scale_factor = camera.scale_factor(distance);

            if (star.radius * star.radius * scale_factor * scale_factor
                >= settings.min_size_for_rays)
                || distance < 0.0
            {
                self.star_data_rays.push(StarBufferRays {
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
            } else {
                self.star_instance_data.push(StarInstanceBuffer {
                    pos: transform.translation.extend(star.radius),
                    color: star.color,
                    shift: star.shift,
                });
            }
        }

        let uniform_buffer_align = self
            .render
            .device()
            .limits()
            .min_uniform_buffer_offset_alignment as usize;

        let size = Layout::new::<StarBufferRays>().pad_to_align().size();
        let offset = crate::utils::align(size, uniform_buffer_align);

        if self.star_data_rays.len() > self.star_bind_groups_rays.len() {
            self.star_buffer_rays = self.render.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("Star Data Buffer Rays"),
                size: (offset * self.star_data_rays.len()) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

            self.star_bind_groups_rays.clear();

            for i in 0..self.star_data_rays.len() {
                self.star_bind_groups_rays
                    .push(
                        self.render
                            .device()
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Star Bind Group Rays"),
                                layout: &self.star_bind_group_layout_rays,
                                entries: &[wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer: &self.star_buffer_rays,
                                        offset: (offset * i) as u64,
                                        size: Some(NonZeroU64::new(size as u64).unwrap()),
                                    }),
                                }],
                            }),
                    );
            }
        }

        for i in 0..self.star_data_rays.len() {
            self.render.queue().write_buffer(
                &self.star_buffer_rays,
                (i * offset) as u64,
                bytemuck::cast_slice(&self.star_data_rays[i..i + 1]),
            );
        }

        if self.star_instance_data.len() > self.star_instance_count {
            let layout = Layout::new::<StarInstanceBuffer>();

            self.star_instance_count = self.star_instance_data.len();
            self.star_instance_buffer =
                self.render.device().create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Star Instance Buffer"),
                    size: (layout.size() * self.star_instance_count) as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    mapped_at_creation: false,
                });
        }

        self.render.queue().write_buffer(
            &self.star_instance_buffer,
            0,
            bytemuck::cast_slice(self.star_instance_data.as_slice()),
        );
    }

    pub fn render<'s>(&'s self, render_pass: &mut wgpu::RenderPass<'s>, camera: &Camera) {
        // "Particle" rendering
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
        render_pass.set_bind_group(0, &self.env_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.star_instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.star_instance_data.len() as u32);

        // Ray Marching
        render_pass.set_pipeline(&self.pipeline_rays);
        render_pass.set_viewport(
            0.0,
            0.0,
            camera.width() as f32,
            camera.height() as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(0, 0, camera.width(), camera.height());
        render_pass.set_bind_group(0, &self.env_bind_group_rays, &[]);

        for i in 0..self.star_data_rays.len() {
            render_pass.set_bind_group(1, &self.star_bind_groups_rays[i], &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct EnvBufferRays {
    inv_proj_view: Mat4,
    proj_view: Mat4,
    /// x, y, z = position
    camera: Vec4,
    near: f32,
    far: f32,
    time: f32,
}

unsafe impl Pod for EnvBufferRays {}

unsafe impl Zeroable for EnvBufferRays {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct StarBufferRays {
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

unsafe impl Pod for StarBufferRays {}

unsafe impl Zeroable for StarBufferRays {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct EnvBuffer {
    proj: Mat4,
    view: Mat4,
}

unsafe impl Pod for EnvBuffer {}

unsafe impl Zeroable for EnvBuffer {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct StarInstanceBuffer {
    pos: Vec4,
    color: Vec4,
    shift: Vec4,
}

unsafe impl Pod for StarInstanceBuffer {}

unsafe impl Zeroable for StarInstanceBuffer {}

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
