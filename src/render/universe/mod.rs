use starlight::World;

use super::{Camera, RenderCtxRef};

mod bloom;
mod star;

pub use bloom::{BloomCompute, BloomSettings};
pub use star::{StarPipeline, StarSettings};

pub struct RendererSettings {
    pub bloom: BloomSettings,
    pub star: StarSettings,
    pub exposure: f32,
}

pub struct UniverseRenderer {
    render: RenderCtxRef,
    star_pipeline: StarPipeline,
    bloom_compute: BloomCompute,

    width: u32,
    height: u32,

    main_texture: wgpu::Texture,
    main_view: wgpu::TextureView,

    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    composite_bind_group_layout: wgpu::BindGroupLayout,
    composite_pipeline: wgpu::RenderPipeline,
    composite_data: CompositeBuffer,
    composite_buffer: wgpu::Buffer,
    composite_sampler: wgpu::Sampler,
}

impl UniverseRenderer {
    pub fn new(render: RenderCtxRef) -> Self {
        let star_pipeline = StarPipeline::new(render.clone());
        let bloom_compute = BloomCompute::new(render.clone());

        let main_texture = render.create_hdr_attachment(1, 1, 1, true);
        let main_view = main_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = render.create_depth_attachment(1, 1, 1);

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // Composite Pipeline

        let composite_module = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/composite.wgsl"));

        let composite_bind_group_layout =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Composite Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        // wgpu::BindGroupLayoutEntry {
                        //     binding: 2,
                        //     visibility: wgpu::ShaderStages::FRAGMENT,
                        //     ty: wgpu::BindingType::Texture {
                        //         multisampled: false,
                        //         sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        //         view_dimension: wgpu::TextureViewDimension::D2,
                        //     },
                        //     count: None,
                        // },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                has_dynamic_offset: false,
                                min_binding_size: None,
                                ty: wgpu::BufferBindingType::Uniform,
                            },
                            count: None,
                        },
                    ],
                });

        let composite_pipeline_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("composite_pipeline_layout"),
                    bind_group_layouts: &[&composite_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let composite_pipeline =
            render
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("composite_pipeline"),
                    layout: Some(&composite_pipeline_layout),
                    vertex: wgpu::VertexState {
                        entry_point: "vs_main",
                        module: &composite_module,
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        alpha_to_coverage_enabled: false,
                        count: 1,
                        mask: !0,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &composite_module,
                        entry_point: "fs_main",
                        targets: &[wgpu::ColorTargetState {
                            format: render.ldr_format(),
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        }],
                    }),
                });

        let composite_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Composite Settings Buffer"),
            size: std::mem::size_of::<CompositeBuffer>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let composite_sampler = render.device().create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: std::f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        Self {
            render,
            star_pipeline,
            bloom_compute,

            width: 1,
            height: 1,
            main_texture,
            main_view,
            depth_texture,
            depth_view,

            composite_bind_group_layout,
            composite_pipeline,
            composite_data: CompositeBuffer {
                exposure: 1.0,
                bloom_intensity: 0.0,
                bloom_dirt_intensity: 0.0,
            },
            composite_buffer,
            composite_sampler,
        }
    }

    pub fn render(&mut self, world: &World, camera: &Camera, settings: &RendererSettings) {
        self.star_pipeline.update(world, camera, &settings.star);

        if camera.width() != self.width || camera.height() != self.height {
            self.width = camera.width();
            self.height = camera.height();

            self.main_texture = self
                .render
                .create_hdr_attachment(self.width, self.height, 1, true);
            self.main_view = self
                .main_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.depth_texture = self
                .render
                .create_depth_attachment(self.width, self.height, 1);
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }

        let mut encoder =
            self.render
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Universe Command Encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.main_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        self.star_pipeline.render(&mut render_pass, camera);

        drop(render_pass);

        self.bloom_compute.compute(
            &mut encoder,
            &self.main_view,
            self.width,
            self.height,
            &settings.bloom,
        );

        // ************************
        // Composite Pass *********
        // ************************

        self.composite_data.exposure = settings.exposure;
        self.composite_data.bloom_intensity = settings.bloom.intensity;
        self.composite_data.bloom_dirt_intensity = 0.0;

        // Update settings
        self.render.queue().write_buffer(
            &self.composite_buffer,
            0,
            bytemuck::cast_slice(&[self.composite_data]),
        );

        let composite_bind_group =
            self.render
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Composite Bind Group"),
                    layout: &self.composite_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.main_view),
                            // resource: wgpu::BindingResource::TextureView(self.bloom_compute.view()),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(self.bloom_compute.view()),
                            // resource: wgpu::BindingResource::TextureView(&self.main_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.composite_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.composite_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Composite Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: camera.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.composite_pipeline);
        render_pass.set_viewport(
            0.0,
            0.0,
            camera.width() as f32,
            camera.height() as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(0, 0, camera.width(), camera.height());
        render_pass.set_bind_group(0, &composite_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        drop(render_pass);

        self.render
            .queue()
            .submit(std::iter::once(encoder.finish()));
    }
}

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct CompositeBuffer {
    exposure: f32,
    bloom_intensity: f32,
    bloom_dirt_intensity: f32,
}

unsafe impl Pod for CompositeBuffer {}

unsafe impl Zeroable for CompositeBuffer {}
