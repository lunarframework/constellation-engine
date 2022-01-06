use super::RenderCtxRef;
use std::num::NonZeroU32;

const BLOOM_COMPUTE_WORKSIZE: u32 = 4;

pub struct BloomCompute {
    render: RenderCtxRef,

    width: u32,
    height: u32,
    levels: u32,

    needs_resize: bool,

    sampler: wgpu::Sampler,

    // bloom_dirt_texture: wgpu::Texture,
    // bloom_dirt_view: wgpu::TextureView,

    // Downsampling
    downsample_texture: wgpu::Texture,
    downsample_view: wgpu::TextureView,
    downsample_mip_views: Vec<wgpu::TextureView>,

    ping_texture: wgpu::Texture,
    ping_view: wgpu::TextureView,
    ping_mip_views: Vec<wgpu::TextureView>,

    down_pipeline: wgpu::ComputePipeline,
    down_bind_group_layout: wgpu::BindGroupLayout,

    down_buffers: Vec<wgpu::Buffer>,
    down_data: Vec<DownUniformBuffer>,
    down_bind_groups: Vec<wgpu::BindGroup>,

    // Upsampling
    upsample_texture: wgpu::Texture,
    upsample_view: wgpu::TextureView,
    upsample_mip_views: Vec<wgpu::TextureView>,

    up_pipeline: wgpu::ComputePipeline,
    up_bind_group_layout: wgpu::BindGroupLayout,

    up_buffers: Vec<wgpu::Buffer>,
    up_data: Vec<UpUniformBuffer>,
    up_bind_groups: Vec<wgpu::BindGroup>,
}

impl BloomCompute {
    pub fn new(render: RenderCtxRef) -> Self {
        // let bloom_dirt_texture = render.device().create_texture(&wgpu::TextureDescriptor {
        //     label: Some("Bloom Dirt Texture"),
        //     dimension: wgpu::TextureDimension::D2,
        //     format: render.ldr_format(),
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        //     size: wgpu::Extent3d {
        //         width: 1,
        //         height: 1,
        //         depth_or_array_layers: 1,
        //     },
        // });

        // Down Pipeline

        let downsample_texture = render.default_texture();

        let downsample_view =
            downsample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let ping_texture = render.default_texture();

        let ping_view = ping_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // let uniform_buffer = render.device().create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Bloom Uniform Buffer"),
        //     size: std::mem::size_of::<UniformBuffer>() as u64,
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        let down_bind_group_layout =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: render.hdr_format(),
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                has_dynamic_offset: false,
                                min_binding_size: None,
                                ty: wgpu::BufferBindingType::Uniform,
                            },
                            count: None,
                        },
                    ],
                });

        // let bind_group = render
        //     .device()
        //     .create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some("Star Env Bind Group"),
        //         layout: &down_bind_group_layout,
        //         entries: &[wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                 buffer: &uniform_buffer,
        //                 offset: 0,
        //                 size: None,
        //             }),
        //         }],
        //     });

        let down_pipeline_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Pipeline Layout"),
                    bind_group_layouts: &[&down_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let down_module = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/bloom_down.wgsl"));

        let down_pipeline =
            render
                .device()
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Bloom Pipeline"),
                    entry_point: "main",
                    layout: Some(&down_pipeline_layout),
                    module: &down_module,
                });

        let upsample_texture = render.default_texture();

        let upsample_view = upsample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let up_bind_group_layout =
            render
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: render.hdr_format(),
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                has_dynamic_offset: false,
                                min_binding_size: None,
                                ty: wgpu::BufferBindingType::Uniform,
                            },
                            count: None,
                        },
                    ],
                });

        let up_pipeline_layout =
            render
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Pipeline Layout"),
                    bind_group_layouts: &[&up_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let up_module = render
            .device()
            .create_shader_module(&wgpu::include_wgsl!("shaders/bloom_up.wgsl"));

        let up_pipeline =
            render
                .device()
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Bloom Upsampling Pipeline"),
                    entry_point: "main",
                    layout: Some(&up_pipeline_layout),
                    module: &up_module,
                });

        let sampler = render.device().create_sampler(&wgpu::SamplerDescriptor {
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

        // let bind_group = render
        //     .device()
        //     .create_bind_group(&wgpu::BindGroupDescriptor {
        //         label: Some("Star Env Bind Group"),
        //         layout: &down_bind_group_layout,
        //         entries: &[wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //                 buffer: &uniform_buffer,
        //                 offset: 0,
        //                 size: None,
        //             }),
        //         }],
        //     });

        // let bloom_dirt_view =
        //bloom_dirt_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            render,
            width: 1,
            height: 1,
            levels: 1,
            needs_resize: true,

            sampler,

            downsample_texture,
            downsample_view,
            downsample_mip_views: Vec::new(),

            ping_texture,
            ping_view,
            ping_mip_views: Vec::new(),

            down_pipeline,
            down_bind_group_layout,

            down_data: Vec::new(),
            down_buffers: Vec::new(),
            down_bind_groups: Vec::new(),

            upsample_texture,
            upsample_view,
            upsample_mip_views: Vec::new(),

            up_pipeline,
            up_bind_group_layout,

            up_data: Vec::new(),
            up_buffers: Vec::new(),
            up_bind_groups: Vec::new(),
        }
    }

    pub fn compute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        let width = width / 2;
        let height = height / 2;

        let width = width + (BLOOM_COMPUTE_WORKSIZE - (width % BLOOM_COMPUTE_WORKSIZE));
        let height = height + (BLOOM_COMPUTE_WORKSIZE - (height % BLOOM_COMPUTE_WORKSIZE));

        self.resize(width, height);
        self.update(view);

        // Pre filtering

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Bloom Pass"),
        });

        compute_pass.set_pipeline(&self.down_pipeline);
        // Set threshold to threashold
        // Knee to knee
        // lod to 0.0
        // mode to 0
        // compute_pass.set_bind_group(...);

        let mut uniform_index = 0;

        let mut workgroup_x = self.width / BLOOM_COMPUTE_WORKSIZE;
        let mut workgroup_y = self.height / BLOOM_COMPUTE_WORKSIZE;

        compute_pass.set_bind_group(0, &self.down_bind_groups[uniform_index], &[]);
        compute_pass.dispatch(workgroup_x, workgroup_y, 1);
        uniform_index += 1;

        let base_size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let mips = self.levels - 2;

        // Fill downsample mips 1 through levels - 2

        for i in 1..mips {
            let mip_size = base_size.mip_level_size(i, false);

            workgroup_x = ((mip_size.width / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;
            workgroup_y = ((mip_size.height / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;

            compute_pass.set_bind_group(0, &self.down_bind_groups[uniform_index], &[]);
            compute_pass.dispatch(workgroup_x, workgroup_y, 1);
            uniform_index += 1;

            compute_pass.set_bind_group(0, &self.down_bind_groups[uniform_index], &[]);
            compute_pass.dispatch(workgroup_x, workgroup_y, 1);
            uniform_index += 1;
        }

        // Now downsample[0] = prefiltered scene
        // And each mip contains a downsampled version of the previous mip

        compute_pass.set_pipeline(&self.up_pipeline);

        let mut uniform_index = 0;

        if mips > 2 {
            let mip_size = base_size.mip_level_size(mips - 2, false);

            workgroup_x = ((mip_size.width / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;
            workgroup_y = ((mip_size.height / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;
            compute_pass.set_bind_group(0, &self.up_bind_groups[uniform_index], &[]);
            compute_pass.dispatch(workgroup_x, workgroup_y, 1);
            uniform_index += 1;

            for i in (0..(mips - 2)).rev() {
                let mip_size = base_size.mip_level_size(i, false);

                workgroup_x = ((mip_size.width / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;
                workgroup_y = ((mip_size.height / BLOOM_COMPUTE_WORKSIZE) as f32).ceil() as u32;

                compute_pass.set_bind_group(0, &self.up_bind_groups[uniform_index], &[]);
                compute_pass.dispatch(workgroup_x, workgroup_y, 1);
                uniform_index += 1;
            }
        }

        for i in 0..uniform_index {
            self.render.queue().write_buffer(
                &self.up_buffers[i],
                0,
                bytemuck::cast_slice(&self.up_data[i..(i + 1)]),
            )
        }

        drop(compute_pass);
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.upsample_texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.upsample_view
    }

    fn resize(&mut self, width: u32, height: u32) {
        if (width != self.width || height != self.height || self.needs_resize)
            && (width > 0 && height > 0)
        {
            self.width = width;
            self.height = height;
            self.needs_resize = false;

            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let mip_level_count = size.max_mips();

            self.levels = mip_level_count;

            self.downsample_texture =
                self.render
                    .device()
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("Bloom Downscale Texture"),
                        dimension: wgpu::TextureDimension::D2,
                        format: self.render.hdr_format(),
                        mip_level_count,
                        sample_count: 1,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::STORAGE_BINDING,
                        size,
                    });

            self.downsample_view = self
                .downsample_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.downsample_mip_views.clear();
            self.downsample_mip_views.reserve(self.levels as usize);

            for i in 0..self.levels {
                self.downsample_mip_views
                    .push(
                        self.downsample_texture
                            .create_view(&wgpu::TextureViewDescriptor {
                                label: None,
                                format: None,
                                dimension: None,
                                aspect: wgpu::TextureAspect::All,
                                base_mip_level: i,
                                mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                                base_array_layer: 0,
                                array_layer_count: Some(NonZeroU32::new(1).unwrap()),
                            }),
                    );
            }

            self.ping_texture = self
                .render
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Bloom Downscale Texture"),
                    dimension: wgpu::TextureDimension::D2,
                    format: self.render.hdr_format(),
                    mip_level_count,
                    sample_count: 1,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    size,
                });

            self.ping_view = self
                .ping_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.ping_mip_views.clear();
            self.ping_mip_views.reserve(self.levels as usize);

            for i in 0..self.levels {
                self.ping_mip_views.push(self.ping_texture.create_view(
                    &wgpu::TextureViewDescriptor {
                        label: None,
                        format: None,
                        dimension: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: i,
                        mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                        base_array_layer: 0,
                        array_layer_count: Some(NonZeroU32::new(1).unwrap()),
                    },
                ));
            }

            let mut downsample_passes = 1;

            if self.levels > 3 {
                downsample_passes += 2 * (self.levels - 3);
            }

            if self.down_data.len() < downsample_passes as usize {
                let additional = downsample_passes as usize - self.down_data.len();
                self.down_data.reserve(additional);
                self.down_buffers.reserve(additional);

                let data = DownUniformBuffer {
                    threshold: 1.0,
                    knee: 0.1,
                    lod: 0.0,
                    filter: 0,
                };

                let buffer = self.render.device().create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    mapped_at_creation: false,
                    size: std::mem::size_of::<DownUniformBuffer>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                self.down_data.push(data);
                self.down_buffers.push(buffer);
            }

            self.upsample_texture = self
                .render
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Bloom Downscale Texture"),
                    dimension: wgpu::TextureDimension::D2,
                    format: self.render.hdr_format(),
                    mip_level_count,
                    sample_count: 1,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING,
                    size,
                });

            self.upsample_view = self
                .upsample_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.upsample_mip_views.clear();
            self.upsample_mip_views.reserve(self.levels as usize);

            for i in 0..self.levels {
                self.upsample_mip_views
                    .push(
                        self.upsample_texture
                            .create_view(&wgpu::TextureViewDescriptor {
                                label: None,
                                format: None,
                                dimension: None,
                                aspect: wgpu::TextureAspect::All,
                                base_mip_level: i,
                                mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                                base_array_layer: 0,
                                array_layer_count: Some(NonZeroU32::new(1).unwrap()),
                            }),
                    );
            }

            let mut upsample_passes = 1;

            if self.levels > 4 {
                upsample_passes += self.levels - 4;
            }

            if self.up_data.len() < upsample_passes as usize {
                let additional = upsample_passes as usize - self.up_data.len();
                self.up_data.reserve(additional);
                self.up_buffers.reserve(additional);

                let data = UpUniformBuffer { lod: 0.0 };

                let buffer = self.render.device().create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    mapped_at_creation: false,
                    size: std::mem::size_of::<UpUniformBuffer>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                self.up_data.push(data);
                self.up_buffers.push(buffer);
            }
        }
    }

    fn update(&mut self, view: &wgpu::TextureView) {
        let mut uniform_index = 0;
        self.down_bind_groups.clear();

        self.down_data[uniform_index].threshold = 1.0;
        self.down_data[uniform_index].knee = 0.1;
        self.down_data[uniform_index].lod = 0.0;
        self.down_data[uniform_index].filter = 1;

        let bind_group = self
            .render
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.down_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.downsample_mip_views[0]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.down_buffers[uniform_index],
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });
        self.down_bind_groups.push(bind_group);
        uniform_index += 1;

        let mips = self.levels - 2;

        // Fill downsample mips 1 through levels - 2

        for i in 1..mips {
            self.down_data[uniform_index].threshold = 1.0;
            self.down_data[uniform_index].knee = 0.1;
            self.down_data[uniform_index].lod = i as f32 - 1.0;
            self.down_data[uniform_index].filter = 0;

            let bind_group = self
                .render
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.down_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.ping_mip_views[i as usize],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&self.downsample_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.down_buffers[uniform_index],
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                });

            self.down_bind_groups.push(bind_group);
            uniform_index += 1;

            self.down_data[uniform_index].threshold = 1.0;
            self.down_data[uniform_index].knee = 0.1;
            self.down_data[uniform_index].lod = i as f32;
            self.down_data[uniform_index].filter = 0;

            let bind_group = self
                .render
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.down_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.downsample_mip_views[i as usize],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&self.ping_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.down_buffers[uniform_index],
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                });

            self.down_bind_groups.push(bind_group);
            uniform_index += 1;
        }

        for i in 0..uniform_index {
            self.render.queue().write_buffer(
                &self.down_buffers[i],
                0,
                bytemuck::cast_slice(&self.down_data[i..(i + 1)]),
            )
        }

        // Now downsample[0] = prefiltered scene
        // And each mip contains a downsampled version of the previous mip

        let mut uniform_index = 0;
        self.up_bind_groups.clear();

        self.up_data[uniform_index].lod = mips as f32 - 2.0;

        if mips > 2 {
            let bind_group = self
                .render
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.up_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.upsample_mip_views[(mips - 2) as usize],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&self.downsample_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&self.downsample_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.up_buffers[uniform_index],
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                });

            // Set lod to mips - 2

            // Output: mip - 2 th level of upsample
            // Input: downsample[0]
            // Upsample: downsample[0]

            self.up_bind_groups.push(bind_group);
            uniform_index += 1;

            for i in (0..(mips - 2)).rev() {
                self.up_data[uniform_index].lod = i as f32;

                let bind_group =
                    self.render
                        .device()
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.up_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &self.upsample_mip_views[i as usize],
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(
                                        &self.downsample_view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(
                                        &self.upsample_mip_views[(i + 1) as usize],
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer: &self.up_buffers[uniform_index],
                                        offset: 0,
                                        size: None,
                                    }),
                                },
                            ],
                        });
                self.up_bind_groups.push(bind_group);
            }
        }

        for i in 0..uniform_index {
            self.render.queue().write_buffer(
                &self.up_buffers[i],
                0,
                bytemuck::cast_slice(&self.up_data[i..(i + 1)]),
            )
        }
    }
}

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct DownUniformBuffer {
    threshold: f32,
    knee: f32,
    lod: f32,
    filter: u32,
}

unsafe impl Pod for DownUniformBuffer {}

unsafe impl Zeroable for DownUniformBuffer {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct UpUniformBuffer {
    lod: f32,
}

unsafe impl Pod for UpUniformBuffer {}

unsafe impl Zeroable for UpUniformBuffer {}
