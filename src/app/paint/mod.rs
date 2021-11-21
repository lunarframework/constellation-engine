use log::info;
use wgpu::{
    include_wgsl, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Extent3d,
    FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat, Label,
    LoadOp, MultisampleState, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerDescriptor,
    ShaderModule, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use super::{Canvas, Renderer};

/// Id representing a palette. Created with `create_palette`.
#[derive(Copy, Clone, Debug)]
pub struct PaletteId(usize);

impl From<imgui::TextureId> for PaletteId {
    fn from(id: imgui::TextureId) -> PaletteId {
        PaletteId(id.id())
    }
}

impl PaletteId {
    pub fn as_image_id(&self) -> imgui::TextureId {
        imgui::TextureId::from(self.0)
    }
}

#[derive(Clone, Debug)]
pub enum PaintError {
    InvalidPalette(PaletteId),
}

impl fmt::Display for PaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PaintError::InvalidPalette(id) => {
                write!(f, "Painter error: invalid palette id '{}'", id.0)
            }
        }
    }
}

impl Error for PaintError {}

/// Object responsible for painting texture quads onto a canvas.
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

    index_buffer: Buffer,
    index_buffer_size: u64,
    vertex_buffer: Buffer,
    vertex_buffer_size: u64,

    palettes: Arc<Mutex<PaletteRegistry>>,
}

impl Painter {
    /// Creates painter from renderer.
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

        // Initializes a vertex buffer of size 1 MiB.
        let vertex_buffer_size = 1024 * 1024;

        let vertex_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter Vertex Buffer"),
            size: vertex_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Initializes an index buffer of size 1 MiB.
        let index_buffer_size = 1024 * 1024;

        let index_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter Index Buffer"),
            size: index_buffer_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
            vertex_buffer,
            vertex_buffer_size,
            index_buffer,
            index_buffer_size,
            palettes: Arc::new(Mutex::new(PaletteRegistry::new())),
        }
    }

    /// Creates a new palette using the palette descriptor
    pub fn create_palette(&self, desc: &PaletteDescriptor) -> PaletteId {
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

        self.palettes.lock().unwrap().insert(Palette {
            texture,
            view,
            bind_group,
            width: desc.width,
            height: desc.height,
            is_srgb: desc.srgb,
            is_renderable: desc.renderable,
        })
    }

    pub fn release_palette(&self, id: PaletteId) -> Option<Palette> {
        self.palettes.lock().unwrap().remove(id)
    }

    pub fn write_palette(&self, id: PaletteId, data: &[u8]) {
        let registry = self.palettes.lock().unwrap();
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

    /// Paints the data described in `paint_data` into the canvas.
    pub fn paint(&mut self, canvas: &Canvas, paint_data: &PaintData) -> Result<(), PaintError> {
        if self.output_format != canvas.format() || self.pipeline.is_none() {
            self.output_format = canvas.format();
            self.recreate_pipeline();
        }

        let fb_width = paint_data.size[0] * canvas.scale_factor() as f32;
        let fb_height = paint_data.size[1] * canvas.scale_factor() as f32;

        // // If the render area is <= 0, exit here and now.
        // if !(fb_width > 0.0 && fb_height > 0.0) {
        //     return Ok(());
        // }

        self.update_buffers(paint_data);

        let output = canvas.acquire();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Paint Command Encoder"),
                });

        {
            let palettes = self.palettes.lock().unwrap();
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_buffer_bind_group, &[]);

            for elem in paint_data.elements.iter() {
                let clip_rect = [
                    (elem.clip_rect[0] - paint_data.pos[0]) * canvas.scale_factor() as f32,
                    (elem.clip_rect[1] - paint_data.pos[1]) * canvas.scale_factor() as f32,
                    (elem.clip_rect[2] - paint_data.pos[0]) * canvas.scale_factor() as f32,
                    (elem.clip_rect[3] - paint_data.pos[1]) * canvas.scale_factor() as f32,
                ];

                let palette_id = elem.palette;

                render_pass.set_bind_group(
                    1,
                    &palettes
                        .get(palette_id)
                        .ok_or(PaintError::InvalidPalette(palette_id))?
                        .bind_group,
                    &[],
                );

                // Set scissors on the renderpass.
                if clip_rect[0] < fb_width
                    && clip_rect[1] < fb_height
                    && clip_rect[2] >= 0.0
                    && clip_rect[3] >= 0.0
                {
                    let scissors = (
                        clip_rect[0].max(0.0).floor() as u32,
                        clip_rect[1].max(0.0).floor() as u32,
                        (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                        (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                    );

                    // XXX: Work-around for wgpu issue [1] by only issuing draw
                    // calls if the scissor rect is valid (by wgpu's flawed
                    // logic). Regardless, a zero-width or zero-height scissor
                    // is essentially a no-op render anyway, so just skip it.
                    // [1]: https://github.com/gfx-rs/wgpu/issues/1750
                    if scissors.2 > 0 && scissors.3 > 0 {
                        render_pass
                            .set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);

                        // Draw the current batch of vertices with the renderpass.
                        render_pass.draw_indexed(
                            elem.idx_offset as u32..(elem.idx_offset + elem.idx_count) as u32,
                            elem.vtx_offset as i32,
                            0..1,
                        );
                    }
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.renderer
            .queue()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    fn update_buffers(&mut self, paint_data: &PaintData) {
        let width = paint_data.size[0];
        let height = paint_data.size[1];

        let offset_x = paint_data.pos[0] / width;
        let offset_y = paint_data.pos[1] / height;

        // Create and update the transform matrix for the current frame.
        // This is required to adapt to vulkan coordinates.
        // let matrix = [
        //     [2.0 / width, 0.0, 0.0, 0.0],
        //     [0.0, 2.0 / height as f32, 0.0, 0.0],
        //     [0.0, 0.0, -1.0, 0.0],
        //     [-1.0, -1.0, 0.0, 1.0],
        // ];
        let matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0 - offset_x * 2.0, 1.0 + offset_y * 2.0, 0.0, 1.0],
        ];

        self.renderer
            .queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&matrix));

        let vertex_buffer_size = paint_data.vtx_buffer_bytes() as u64;
        let index_buffer_size = paint_data.idx_buffer_bytes() as u64;

        if self.vertex_buffer_size < vertex_buffer_size {
            self.vertex_buffer = self.renderer.device().create_buffer(&BufferDescriptor {
                label: Some("Painter Vertex Buffer"),
                size: vertex_buffer_size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_size = vertex_buffer_size;
        }

        self.renderer.queue().write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&paint_data.vtx_buffer[..]),
        );

        if self.index_buffer_size < index_buffer_size {
            self.index_buffer = self.renderer.device().create_buffer(&BufferDescriptor {
                label: Some("Painter Index Buffer"),
                size: index_buffer_size,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.index_buffer_size = index_buffer_size;
        }

        self.renderer.queue().write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&paint_data.idx_buffer[..]),
        );
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

pub struct PaintData {
    pub pos: [f32; 2],
    pub size: [f32; 2],

    vtx_buffer: Vec<PaintVtx>,
    idx_buffer: Vec<PaintIdx>,
    elements: Vec<PaintElement>,
}

impl PaintData {
    pub fn new() -> Self {
        Self {
            pos: [0.0; 2],
            size: [0.0; 2],
            vtx_buffer: Default::default(),
            idx_buffer: Default::default(),
            elements: Default::default(),
        }
    }

    pub fn vtx_buffer_bytes(&self) -> usize {
        self.vtx_buffer.len() * std::mem::size_of::<PaintVtx>()
    }

    pub fn idx_buffer_bytes(&self) -> usize {
        self.idx_buffer.len() * std::mem::size_of::<PaintVtx>()
    }

    pub fn reserve(&mut self, vtx_count: usize, idx_count: usize, elem_count: usize) {
        self.vtx_buffer.clear();
        self.vtx_buffer.reserve(vtx_count);
        self.idx_buffer.clear();
        self.idx_buffer.reserve(idx_count);
        self.elements.clear();
        self.elements.reserve(elem_count);
    }

    // pub fn vtx_buffer(&self) -> &[PaintVtx] {
    //     &self.vtx_buffer[..]
    // }

    // pub fn idx_buffer(&self) -> &[PaintIdx] {
    //     &self.idx_buffer[..]
    // }

    // pub fn add_element(
    //     &mut self,
    //     clip_rect: [f32; 4],
    //     palette: PaletteId,
    //     index_offset: usize,
    //     indices: &[PaintIdx],
    //     vertices: &[PaintVtx],
    // ) {

    // }
}

pub struct PaintElement {
    pub idx_count: usize,
    pub clip_rect: [f32; 4],
    pub palette: PaletteId,
    pub vtx_offset: usize,
    pub idx_offset: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PaintVtx {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

unsafe impl bytemuck::Pod for PaintVtx {}
unsafe impl bytemuck::Zeroable for PaintVtx {}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PaintIdx(u16);

unsafe impl bytemuck::Pod for PaintIdx {}
unsafe impl bytemuck::Zeroable for PaintIdx {}

pub struct PaletteDescriptor<'a> {
    pub label: Label<'a>,
    pub width: u32,
    pub height: u32,
    pub srgb: bool,
    pub renderable: bool,
}

pub struct Palette {
    texture: Texture,
    view: TextureView,
    bind_group: BindGroup,
    width: u32,
    height: u32,
    is_srgb: bool,
    is_renderable: bool,
}

impl Palette {
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
struct PaletteRegistry {
    palettes: HashMap<usize, Palette>,
    next: usize,
}

impl PaletteRegistry {
    // TODO: hasher like rustc_hash::FxHashMap or something would let this be
    // `const fn`
    fn new() -> Self {
        Self {
            palettes: HashMap::new(),
            next: 1, // 0 should always be an invalid key
        }
    }

    fn insert(&mut self, palette: Palette) -> PaletteId {
        let id = self.next;
        self.palettes.insert(id, palette);
        self.next += 1;
        PaletteId(id)
    }

    fn replace(&mut self, id: PaletteId, palette: Palette) -> Option<Palette> {
        self.palettes.insert(id.0, palette)
    }

    fn remove(&mut self, id: PaletteId) -> Option<Palette> {
        self.palettes.remove(&id.0)
    }

    fn get(&self, id: PaletteId) -> Option<&Palette> {
        self.palettes.get(&id.0)
    }

    fn get_mut(&mut self, id: PaletteId) -> Option<&mut Palette> {
        self.palettes.get_mut(&id.0)
    }
}

struct ResizableBuffer {
    buffer: Buffer,
    size: usize,
}
