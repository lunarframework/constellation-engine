use bevy::prelude::*;
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
    render_resource::std140::{AsStd140, Std140},
    render_resource::*,
    renderer::{RenderContext, RenderDevice, RenderQueue},
    texture::BevyDefault,
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
};

use super::Star;

pub fn prepare_pipelines(
    msaa: Res<Msaa>,
    mut pipeline: ResMut<StarPipeline>,
    mut pipelines: ResMut<SpecializedPipelines<StarPipeline>>,
    mut cache: ResMut<RenderPipelineCache>,
) {
    let id = pipelines.specialize(
        &mut cache,
        &pipeline,
        StarPipelineKey {
            samples: msaa.samples,
        },
    );
    pipeline.id = id;
}

pub struct StarPipeline {
    pub shader: Handle<Shader>,

    pub id: CachedPipelineId,

    pub env_bind_group_layout: BindGroupLayout,
    pub star_bind_group_layout: BindGroupLayout,

    pub env_bind_group: BindGroup,
    pub env_buffer: Buffer,

    pub star_bind_groups: Vec<BindGroup>,
    pub star_buffers: UniformVec<StarUniform>,
}

impl FromWorld for StarPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let shader = asset_server.load("shaders/star.wgsl");

        let render_device = world.get_resource_mut::<RenderDevice>().unwrap();
        let env_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("env bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(EnvUniform::std140_size_static() as u64),
                    },
                    count: None,
                }],
            });

        let star_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("env bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(StarUniform::std140_size_static() as u64),
                    },
                    count: None,
                }],
            });

        let env_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: EnvUniform::std140_size_static() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let env_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &env_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: env_buffer.as_entire_binding(),
            }],
        });

        Self {
            shader,
            id: CachedPipelineId::INVALID,
            env_bind_group_layout,
            star_bind_group_layout,
            env_bind_group,
            env_buffer,
            star_bind_groups: Vec::new(),
            star_buffers: UniformVec::default(),
        }
    }
}

impl SpecializedPipeline for StarPipeline {
    type Key = StarPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("star pipeline".into()),
            layout: Some(vec![
                self.env_bind_group_layout.clone(),
                self.star_bind_group_layout.clone(),
            ]),
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vs_main".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                unclipped_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: FrontFace::Cw,
                polygon_mode: PolygonMode::Fill,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StarPipelineKey {
    samples: u32,
}

#[derive(AsStd140, Default)]
pub struct EnvUniform {
    pub inv_proj_view: Mat4,
    pub proj_view: Mat4,
    pub position: Vec4,
    pub near: f32,
    pub far: f32,
    pub time: f32,
}

#[derive(AsStd140)]
pub struct StarUniform {
    pub pos: Vec4,
    pub color: Vec4,
    pub shift: Vec4,
    pub granule_lacunarity: f32,
    pub granule_gain: f32,
    pub granule_octaves: f32,
    pub sunspot_sharpness: f32,
    pub sunspot_cutoff: f32,
    pub sunspot_frequency: f32,
}
