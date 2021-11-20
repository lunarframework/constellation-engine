use log::info;
use wgpu::{
    include_wgsl, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Extent3d,
    FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, Label, LoadOp,
    MultisampleState, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModule, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use super::{Canvas, Renderer};

pub type ImageId = imgui::TextureId;

pub struct Painter {
    renderer: Arc<Renderer>,
    pipeline: Option<RenderPipeline>,
    output_format: TextureFormat,
    uniform_buffer: Buffer,
    uniform_buffer_bind_group: BindGroup,
    sampler: Sampler,
    texture_bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader_module: ShaderModule,
    index_buffers: Vec<Buffer>,
    vertex_buffers: Vec<Buffer>,

    images: Arc<Mutex<ImageRegistry>>,
}

impl Painter {
    pub fn new(renderer: Arc<Renderer>) -> Self {
        info!("Initializing Painter");
        let shader_module = renderer
            .device()
            .create_shader_module(&include_wgsl!("shader.wgsl"));

        // Create the uniform matrix buffer.
        let size = 64;
        let uniform_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter uniform buffer"),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the uniform matrix buffer bind group layout.
        let uniform_layout =
            renderer
                .device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // Create the uniform matrix buffer bind group.
        let uniform_buffer_bind_group = renderer.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Painter uniform buffer bind group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create the texture layout for further usage.
        let texture_bind_group_layout =
            renderer
                .device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Painter bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                });

        // Create the render pipeline layout.
        let pipeline_layout = renderer
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Painter pipeline layout"),
                bind_group_layouts: &[&uniform_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create the texture sampler.
        let sampler = renderer.device().create_sampler(&SamplerDescriptor {
            label: Some("Painter sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        Self {
            renderer,
            pipeline: None,
            output_format: TextureFormat::R8Unorm,
            uniform_buffer,
            uniform_buffer_bind_group,
            sampler,
            texture_bind_group_layout,
            pipeline_layout,
            shader_module,
            index_buffers: Vec::new(),
            vertex_buffers: Vec::new(),
            images: Arc::new(Mutex::new(ImageRegistry::new())),
        }
    }

    pub fn create_image(&self, desc: &ImageDescriptor) -> ImageId {
        let mut usage =
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC;

        if desc.renderable {
            usage |= TextureUsages::RENDER_ATTACHMENT;
        }

        // Create the wgpu texture
        let texture = self.renderer.device().create_texture(&TextureDescriptor {
            label: desc.label,
            size: Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: if desc.srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            },
            usage,
        });

        // Extract the texture view.
        let view = texture.create_view(&TextureViewDescriptor::default());

        // Create the texture bind group from the layout.
        let bind_group = self
            .renderer
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: desc.label,
                layout: &self.texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

        self.images.lock().unwrap().insert(Image {
            texture,
            view,
            bind_group,
            width: desc.width,
            height: desc.height,
            is_srgb: desc.srgb,
            is_renderable: desc.renderable,
        })
    }

    pub fn release_image(&self, id: ImageId) -> Option<Image> {
        self.images.lock().unwrap().remove(id)
    }

    pub fn write_image(&self, id: ImageId, data: &[u8]) {
        let registry = self.images.lock().unwrap();
        let image = registry.get(id).unwrap();

        assert_eq!(data.len() as u32, image.width * image.height * 4);

        self.renderer.queue().write_texture(
            // destination (sub)texture
            ImageCopyTexture {
                texture: &image.texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            // source bitmap data
            data,
            // layout of the source bitmap
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(image.width * 4),
                rows_per_image: NonZeroU32::new(image.height),
            },
            // size of the source bitmap
            Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
        );
    }

    // pub fn image_from_id(&self, id: ImageId) -> Option<&Image> {
    //     self.images.lock().unwrap().get(id)
    // }

    // pub fn on_window_event(&mut self, id: WindowId, event: WindowEvent) {
    //     if self.canvas.id() == id {
    //         if let WindowEvent::Resized(size) = event {
    //             if size.width > 0 && size.height > 0 {
    //                 if self.canvas.format() != self
    //             }
    //         }
    //     }
    // }

    pub fn paint(&mut self, canvas: &Canvas) {
        if self.output_format != canvas.format() || self.pipeline.is_none() {
            self.output_format = canvas.format();
            self.recreate_pipeline();
        }

        canvas.present(|output| {
            let view = output.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                self.renderer
                    .device()
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("Paint Command Encoder"),
                    });

            {
                let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Paint Pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
            }

            // submit will accept anything that implements IntoIter
            self.renderer
                .queue()
                .submit(std::iter::once(encoder.finish()));

            true
        });
    }

    fn recreate_pipeline(&mut self) -> RenderPipeline {
        self.renderer
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Painter pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: VertexState {
                    module: &self.shader_module,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 20,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                shader_location: 0,
                                offset: 0,
                                format: VertexFormat::Float32x2,
                            },
                            VertexAttribute {
                                shader_location: 1,
                                offset: 8,
                                format: VertexFormat::Float32x2,
                            },
                            VertexAttribute {
                                shader_location: 2,
                                offset: 16,
                                format: VertexFormat::Unorm8x4,
                            },
                        ],
                    }],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Cw,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    clamp_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    ..Default::default()
                },
                fragment: Some(FragmentState {
                    module: &self.shader_module,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format: self.output_format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::OneMinusDstAlpha,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
            })
    }
}

// pub enum ImageFormat {
//     Rgba8Unorm,
//     Rgba8UnormSrgb,

//     Rgba16Float,

//     Rgba32Float,
// }

// impl ImageFormat {
//     pub fn as_texture_format(&self) -> TextureFormat {
//         match self {
//             Rgba8Unorm => TextureFormat::Rgba8Unorm,
//             Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,

//             Rgba16Float => TextureFormat::Rgba16Float,

//             Rgba32Float => TextureFormat::Rgba32Float,
//         }
//     }

//     pub fn bytes_per_pixel(&self) -> u32 {
//         match self {
//             Rgba8Unorm => 4,
//             Rgba8UnormSrgb => 4,

//             Rgba16Float => 8,

//             Rgba32Float => 16,
//         }
//     }
// }

pub struct ImageDescriptor<'a> {
    pub label: Label<'a>,
    pub width: u32,
    pub height: u32,
    pub srgb: bool,
    pub renderable: bool,
}

pub struct Image {
    texture: Texture,
    view: TextureView,
    bind_group: BindGroup,
    width: u32,
    height: u32,
    is_srgb: bool,
    is_renderable: bool,
}

impl Image {
    /// Write `data` to the texture.
    ///
    /// - `data`: 32-bit (4 bytes per pixel) RGBA bitmap data.
    pub fn write(&self, queue: &Queue, data: &[u8]) {}

    /// The width of the texture in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of the texture in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The number of bytes the texture takes up.
    pub fn size(&self) -> u32 {
        self.width * self.height * 4
    }

    /// Whether or not the image is displayed in srgb space
    pub fn is_srgb(&self) -> bool {
        self.is_srgb
    }

    /// Whether the image can be used as an attachment in a renderpass
    pub fn is_renderable(&self) -> bool {
        self.is_renderable
    }

    /// The underlying `wgpu::Texture`.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// The `wgpu::TextureView` of the underlying texture.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// Generic texture mapping for use by renderers.
struct ImageRegistry {
    images: HashMap<usize, Image>,
    next: usize,
}

impl ImageRegistry {
    // TODO: hasher like rustc_hash::FxHashMap or something would let this be
    // `const fn`
    fn new() -> Self {
        Self {
            images: HashMap::new(),
            next: 1, // 0 should always be an invalid key
        }
    }

    fn insert(&mut self, image: Image) -> ImageId {
        let id = self.next;
        self.images.insert(id, image);
        self.next += 1;
        ImageId::from(id)
    }

    fn replace(&mut self, id: ImageId, image: Image) -> Option<Image> {
        self.images.insert(id.id(), image)
    }

    pub fn remove(&mut self, id: ImageId) -> Option<Image> {
        self.images.remove(&id.id())
    }

    pub fn get(&self, id: ImageId) -> Option<&Image> {
        self.images.get(&id.id())
    }

    pub fn get_mut(&mut self, id: ImageId) -> Option<&mut Image> {
        self.images.get_mut(&id.id())
    }
}
