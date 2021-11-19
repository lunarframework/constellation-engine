use log::info;
use std::sync::Arc;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, FragmentState, FrontFace, LoadOp,
    MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureFormat, TextureSampleType,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

use super::{Canvas, Renderer};

pub struct Painter {
    renderer: Arc<Renderer>,
    pipeline: Option<RenderPipeline>,
    output_format: TextureFormat,
    uniform_buffer: Buffer,
    uniform_buffer_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    shader_module: ShaderModule,
    index_buffers: Vec<Buffer>,
    vertex_buffers: Vec<Buffer>,
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
        Self {
            renderer,
            pipeline: None,
            output_format: TextureFormat::R8Unorm,
            uniform_buffer,
            uniform_buffer_bind_group,
            texture_bind_group_layout,
            pipeline_layout,
            shader_module,
            index_buffers: Vec::new(),
            vertex_buffers: Vec::new(),
        }
    }

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
