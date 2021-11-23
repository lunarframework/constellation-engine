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

use std::borrow::Borrow;
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
    uniform_buffer_bind_group: BindGroup,
    sampler: Sampler,
    texture_bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader_module: ShaderModule,
    buffers: PainterBuffers,
    bind_groups: Arc<Mutex<BindGroupRegistry>>,
}

impl Painter {
    /// Creates painter from renderer.
    pub fn new(renderer: Arc<Renderer>) -> Self {
        info!("Initializing Painter");
        let shader_module = renderer
            .device()
            .create_shader_module(&include_wgsl!("shader.wgsl"));

        let buffers = PainterBuffers::new(renderer.borrow());

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
                resource: buffers.uniform_buffer.as_entire_binding(),
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
            uniform_buffer_bind_group,
            sampler,
            texture_bind_group_layout,
            pipeline_layout,
            shader_module,
            buffers,
            bind_groups: Arc::new(Mutex::new(BindGroupRegistry::new())),
        }
    }

    /// Creates a new palette using the palette descriptor
    pub fn create_palette(&self, desc: &PaletteDescriptor) -> Palette {
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

        let id = self.bind_groups.lock().unwrap().insert(bind_group);

        Palette {
            texture,
            view,
            registry: self.bind_groups.clone(),
            id,
            width: desc.width,
            height: desc.height,
            is_srgb: desc.srgb,
            is_renderable: desc.renderable,
        }
    }

    pub fn write_palette(&self, palette: &Palette, data: &[u8]) {
        assert_eq!(data.len() as u32, palette.width * palette.height * 4);

        self.renderer.queue().write_texture(
            // destination (sub)texture
            ImageCopyTexture {
                texture: &palette.texture,
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: TextureAspect::All,
            },
            // source bitmap data
            data,
            // layout of the source bitmap
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(palette.width * 4),
                rows_per_image: NonZeroU32::new(palette.height),
            },
            // size of the source bitmap
            Extent3d {
                width: palette.width,
                height: palette.height,
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
        // println!("Display size: {:?}", paint_data.size);
        // println!("Canvas size: {:?}", canvas.size());
        // println!("Canvas scale factor: {}", canvas.scale_factor());
        if self.output_format != canvas.format() || self.pipeline.is_none() {
            self.output_format = canvas.format();
            self.recreate_pipeline();
        }

        let fb_width = paint_data.size[0] * canvas.scale_factor() as f32;
        let fb_height = paint_data.size[1] * canvas.scale_factor() as f32;

        //println!("Framebuffer: {:?}", (fb_width, fb_height));

        // // If the render area is <= 0, exit here and now.
        // if !(fb_width > 0.0 && fb_height > 0.0) {
        //     return Ok(());
        // }

        self.buffers.update(self.renderer.borrow(), paint_data);

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
            let bind_groups = self.bind_groups.lock().unwrap();
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
            render_pass.set_index_buffer(self.buffers.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.buffers.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_buffer_bind_group, &[]);

            render_pass.set_viewport(0.0, 0.0, fb_width, fb_height, 0.0, 1.0);

            for elem in paint_data.elements.iter() {
                let clip_min = [
                    (elem.clip_rect[0] - paint_data.pos[0]) * canvas.scale_factor() as f32,
                    (elem.clip_rect[1] - paint_data.pos[1]) * canvas.scale_factor() as f32,
                ];

                let clip_max = [
                    (elem.clip_rect[2] - paint_data.pos[0]) * canvas.scale_factor() as f32,
                    (elem.clip_rect[3] - paint_data.pos[1]) * canvas.scale_factor() as f32,
                ];

                if clip_max[0] >= clip_min[0] && clip_max[1] >= clip_min[1] {
                    let palette_id = elem.palette_id;

                    render_pass.set_bind_group(
                        1,
                        bind_groups
                            .get(palette_id.0)
                            .ok_or(PaintError::InvalidPalette(palette_id))?,
                        &[],
                    );
                    render_pass.set_scissor_rect(
                        clip_min[0].floor() as u32,
                        clip_min[1].floor() as u32,
                        (clip_max[0] - clip_min[0]).ceil() as u32,
                        (clip_max[1] - clip_min[1]).ceil() as u32,
                    );

                    // Draw the current batch of vertices with the renderpass.
                    render_pass.draw_indexed(
                        elem.idx_offset as u32..(elem.idx_offset + elem.idx_count) as u32,
                        elem.vtx_offset as i32,
                        0..1,
                    );
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

    fn recreate_pipeline(&mut self) {
        self.pipeline = Some(self.renderer.device().create_render_pipeline(
            &RenderPipelineDescriptor {
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
            },
        ));
    }
}

#[derive(Debug)]
pub struct PaintData {
    pos: [f32; 2],
    size: [f32; 2],

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

    pub fn set_pos(&mut self, pos: [f32; 2]) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
    }

    pub fn vtx_buffer(&self) -> &[PaintVtx] {
        &self.vtx_buffer[..]
    }

    pub fn idx_buffer(&self) -> &[PaintIdx] {
        &self.idx_buffer[..]
    }

    pub fn reserve(&mut self, vtx_count: usize, idx_count: usize, elem_count: usize) {
        self.vtx_buffer.clear();
        self.vtx_buffer.reserve(vtx_count);
        self.idx_buffer.clear();
        self.idx_buffer.reserve(idx_count);
        self.elements.clear();
        self.elements.reserve(elem_count);
    }

    pub fn add_vtx_sub_buffer(&mut self, buffer: &[PaintVtx]) {
        self.vtx_buffer.extend_from_slice(buffer);
    }

    pub fn add_idx_sub_buffer(&mut self, buffer: &[PaintIdx]) {
        self.idx_buffer.extend_from_slice(buffer);
    }

    pub fn add_element(&mut self, elem: PaintElement) {
        self.elements.push(elem);
    }
}

#[derive(Debug)]
pub struct PaintElement {
    pub idx_count: usize,
    pub clip_rect: [f32; 4],
    pub palette_id: PaletteId,
    pub vtx_offset: usize,
    pub idx_offset: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PaintVtx {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

unsafe impl bytemuck::Pod for PaintVtx {}
unsafe impl bytemuck::Zeroable for PaintVtx {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
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
    registry: Arc<Mutex<BindGroupRegistry>>,
    id: usize,
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

    pub fn id(&self) -> PaletteId {
        PaletteId(self.id)
    }

    pub fn image_id(&self) -> imgui::TextureId {
        imgui::TextureId::from(self.id)
    }
}

impl Drop for Palette {
    fn drop(&mut self) {
        self.registry.lock().unwrap().remove(self.id);
    }
}

/// Generic texture mapping for use by renderers.
struct BindGroupRegistry {
    bind_groups: HashMap<usize, BindGroup>,
    next: usize,
}

impl BindGroupRegistry {
    // TODO: hasher like rustc_hash::FxHashMap or something would let this be
    // `const fn`
    fn new() -> Self {
        BindGroupRegistry {
            bind_groups: HashMap::new(),
            next: 1, // 0 should always be an invalid key
        }
    }

    fn insert(&mut self, bind_group: BindGroup) -> usize {
        let id = self.next;
        self.bind_groups.insert(id, bind_group);
        self.next += 1;
        id
    }

    // fn replace(&mut self, id: usize, bind_group: BindGroup) -> Option<BindGroup> {
    //     self.bind_groups.insert(id, bind_group)
    // }

    fn remove(&mut self, id: usize) -> Option<BindGroup> {
        self.bind_groups.remove(&id)
    }

    fn get(&self, id: usize) -> Option<&BindGroup> {
        self.bind_groups.get(&id)
    }
}

struct PainterBuffers {
    staging_buffer: Vec<u8>,
    vertex_buffer: Buffer,
    vertex_buffer_size: u64,
    index_buffer: Buffer,
    index_buffer_size: u64,
    uniform_buffer: Buffer,
}

impl PainterBuffers {
    fn new(renderer: &Renderer) -> Self {
        let initial_size = 1024 * 1024;
        let uniform_size = 16;
        let staging_buffer = vec![0u8; initial_size as usize];
        let vertex_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter Vertex Buffer"),
            size: initial_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter Index Buffer"),
            size: initial_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = renderer.device().create_buffer(&BufferDescriptor {
            label: Some("Painter Uniform Buffer"),
            size: uniform_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            staging_buffer,
            vertex_buffer,
            vertex_buffer_size: initial_size,
            index_buffer,
            index_buffer_size: initial_size,
            uniform_buffer,
        }
    }

    fn update(&mut self, renderer: &Renderer, paint_data: &PaintData) {
        // let l = paint_data.pos[0];
        // let r = paint_data.pos[0] + paint_data.size[0];
        // let t = paint_data.pos[1];
        // let b = paint_data.pos[1] + paint_data.size[1];

        // let matrix = [
        //     [2.0 / (r - l), 0.0, 0.0, 0.0],
        //     [0.0, 2.0 / (t - b), 0.0, 0.0],
        //     [0.0, 0.0, 1.0, 0.0],
        //     [(r + l) / (r - l), (t + b) / (b - t), 0.5, 1.0],
        // ];
        let scale = [2.0 / paint_data.size[0], -2.0 / paint_data.size[1]];
        let translate = [
            -1.0 - paint_data.pos[0] * scale[0],
            1.0 + paint_data.pos[1] * scale[1],
        ];

        let uniform_data = [scale, translate];

        renderer
            .queue()
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform_data));

        fn apply_copy_alignment(len: usize) -> u64 {
            // Valid vulkan usage is
            // 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
            // 2. buffer size must be greater than 0.
            // Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
            let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
            ((len as u64 + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT)
        }

        use std::mem::size_of;

        let vertex_buffer_size =
            apply_copy_alignment(paint_data.vtx_buffer().len() * size_of::<PaintVtx>());
        let index_buffer_size =
            apply_copy_alignment(paint_data.idx_buffer().len() * size_of::<PaintIdx>());

        if self.vertex_buffer_size < vertex_buffer_size {
            self.vertex_buffer = renderer.device().create_buffer(&BufferDescriptor {
                label: Some("Painter Vertex Buffer"),
                size: vertex_buffer_size,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_size = vertex_buffer_size;
        }

        self.staging_buffer.clear();
        self.staging_buffer.reserve(vertex_buffer_size as usize);
        self.staging_buffer
            .extend_from_slice(bytemuck::cast_slice(paint_data.vtx_buffer()));

        self.staging_buffer.resize(vertex_buffer_size as usize, 0);

        renderer
            .queue()
            .write_buffer(&self.vertex_buffer, 0, &self.staging_buffer[..]);

        if self.index_buffer_size < index_buffer_size {
            self.index_buffer = renderer.device().create_buffer(&BufferDescriptor {
                label: Some("Painter Index Buffer"),
                size: index_buffer_size,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.index_buffer_size = index_buffer_size;
        }

        self.staging_buffer.clear();
        self.staging_buffer.reserve(index_buffer_size as usize);
        self.staging_buffer
            .extend_from_slice(bytemuck::cast_slice(paint_data.idx_buffer()));

        self.staging_buffer.resize(index_buffer_size as usize, 0);

        renderer
            .queue()
            .write_buffer(&self.index_buffer, 0, &self.staging_buffer[..]);
    }
}
