use bevy::prelude::*;
use bevy::render::{
    render_graph::RenderGraph,
    render_resource::std140::{AsStd140, Std140},
    render_resource::*,
    renderer::RenderQueue,
    view::{ExtractedView, ViewDepthTexture, ViewTarget},
    RenderApp, RenderStage,
};
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    renderer::RenderContext,
};

mod normal;

use normal::{prepare_pipelines, EnvUniform, StarPipeline, StarUniform};

pub mod node {
    pub const STAR_NODE: &str = "star";
}

// use render::StarRenderPlugin;

#[derive(Component, Clone)]
pub struct Star {
    pub temp: f32,
}

pub struct StarPlugin;

impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugin(StarRenderPlugin);
        let render_app = app
            .sub_app_mut(RenderApp)
            .init_resource::<StarPipeline>()
            .init_resource::<SpecializedPipelines<StarPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_stars)
            .add_system_to_stage(RenderStage::Prepare, prepare_pipelines);

        let star_node = StarNode::new(&mut render_app.world);

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_node(node::STAR_NODE, star_node);
    }
}

fn extract_stars(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    mut query: Query<(Entity, &Star)>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, star) in query.iter_mut() {
        values.push((entity, (star.clone(),)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

pub struct StarNode {
    query: QueryState<(
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static ExtractedView,
    )>,
}

impl StarNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for StarNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(StarNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let render_queue = world.get_resource::<RenderQueue>().unwrap();
        let pipeline_cache = world.get_resource::<RenderPipelineCache>().unwrap();

        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (target, depth, view) = match self.query.get_manual(world, view_entity) {
            Ok(query) => query,
            Err(_) => return Ok(()), // No window
        };

        let proj_mat = view.projection;
        let view_mat = view.transform.compute_matrix().inverse();
        let proj_view_mat = proj_mat * view_mat;

        // for (entity, transform, star) in world.query::<(Entity, Transform, Star)

        let pipeline = world.get_resource::<StarPipeline>().unwrap();

        let uniform = EnvUniform {
            inv_proj_view: proj_view_mat.inverse(),
            proj_view: proj_view_mat,
            position: view.transform.translation.extend(1.0),
            near: view.near,
            far: view.far,
            time: 0.0,
        };

        render_queue.write_buffer(&pipeline.env_buffer, 0, uniform.as_std140().as_bytes());

        let render_pipeline = pipeline_cache.get(pipeline.id).unwrap();

        let pass_descriptor = RenderPassDescriptor {
            label: Some("star_pass"),
            // NOTE: The opaque pass loads the color
            // buffer as well as writing to it.
            color_attachments: &[target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth.view,
                // NOTE: The opaque main pass loads the depth buffer and possibly overwrites it
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        };

        let mut render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_viewport(0.0, 0.0, view.width as f32, view.height as f32, 0.0, 1.0);
        render_pass.set_scissor_rect(0, 0, view.width, view.height);
        render_pass.set_bind_group(0, &pipeline.env_bind_group, &[]);

        for star_bind_group in pipeline.star_bind_groups.iter() {
            render_pass.set_bind_group(0, star_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        drop(render_pass);

        Ok(())
    }
}
