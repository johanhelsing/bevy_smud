use std::{cmp::Ordering, ops::Range};

use bevy::{
    asset::HandleId,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    math::Vec3Swizzles,
    prelude::*,
    reflect::Uuid,
    render::{
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            CachedRenderPipelineId, PipelineCache, PrimitiveTopology, SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        Extract, RenderApp, RenderStage,
    },
    sprite::Mesh2dPipelineKey,
    ui::TransparentUi,
    utils::FloatOrd,
};
use copyless::VecHelper;

use crate::{
    ExtractedShape, SetShapeViewBindGroup, ShapeMeta, ShapeVertex, SmudPipeline, SmudPipelineKey,
    SmudShape,
};

type DrawSmudUiShape = (SetItemPipeline, SetShapeViewBindGroup<0>, DrawUiShapeNode);

pub struct DrawUiShapeNode;
impl EntityRenderCommand for DrawUiShapeNode {
    type Param = (SRes<ShapeMeta>, SQuery<Read<UiShapeBatch>>);

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (ui_shape_meta, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let batch = query_batch.get(item).unwrap();

        pass.set_vertex_buffer(
            0,
            ui_shape_meta
                .into_inner()
                .ui_vertices
                .buffer()
                .unwrap()
                .slice(..),
        );
        pass.draw(0..4, batch.range.clone());
        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct UiShapePlugin;

impl Plugin for UiShapePlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                // re-using command from regular pass... ok?
                .add_render_command::<TransparentUi, DrawSmudUiShape>()
                .init_resource::<ExtractedUiShapes>()
                .add_system_to_stage(RenderStage::Extract, extract_ui_shapes)
                .add_system_to_stage(RenderStage::Prepare, prepare_ui_shapes)
                .add_system_to_stage(RenderStage::Queue, queue_ui_shapes);
        }
    }
}

#[derive(Resource, Default, Debug)]
struct ExtractedUiShapes(Vec<ExtractedShape>);

#[allow(clippy::type_complexity)]
fn extract_ui_shapes(
    mut extracted_shapes: ResMut<ExtractedUiShapes>,
    query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &SmudShape,
            &Visibility,
            &BackgroundColor,
        )>,
    >,
) {
    extracted_shapes.0.clear();

    for (node, transform, shape, visibility, color) in query.iter() {
        if !visibility.is_visible {
            continue;
        }

        let size = node.size().x; // TODO: Also pass on the height value
        let frame = size / 2.;

        extracted_shapes.0.alloc().init(ExtractedShape {
            color: shape.color * Vec4::from(color.0),
            transform: *transform,
            sdf_shader: shape.sdf.clone_weak(),
            fill_shader: shape.fill.clone_weak(),
            frame,
            params: shape.params,
        });
    }
}

fn prepare_ui_shapes(
    mut commands: Commands,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut extracted_shapes: ResMut<ExtractedUiShapes>,
    mut shape_meta: ResMut<ShapeMeta>, // TODO: make UI meta?
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    render_queue: Res<RenderQueue>,
) {
    shape_meta.ui_vertices.clear();

    let extracted_shapes = &mut extracted_shapes.0;
    // Sort shapes by z for correct transparency and then by handle to improve batching
    extracted_shapes.sort_unstable_by(|a, b| {
        match a
            .transform
            .translation()
            .z
            .partial_cmp(&b.transform.translation().z)
        {
            Some(Ordering::Equal) | None => {
                (&a.sdf_shader, &a.fill_shader).cmp(&(&b.sdf_shader, &b.fill_shader))
            }
            Some(other) => other,
        }
    });

    let shape_meta = &mut shape_meta;

    let mut start = 0;
    let mut end = 0;
    let mut current_batch_shaders = (
        HandleId::Id(Uuid::nil(), u64::MAX),
        HandleId::Id(Uuid::nil(), u64::MAX),
    );
    let mut last_z = 0.;
    let mut current_batch_pipeline = CachedRenderPipelineId::INVALID;

    // todo: how should msaa be handled for ui?
    // would perhaps be solved if I move this to queue?
    let mesh_key = Mesh2dPipelineKey::from_msaa_samples(1)
        | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

    for extracted_shape in extracted_shapes.iter() {
        let shader_key = (
            extracted_shape.sdf_shader.id(),
            extracted_shape.fill_shader.id(),
        );
        let position = extracted_shape.transform.translation();
        let z = position.z;

        // We also split by z, so other ui systems can get their stuff in the middle
        if current_batch_shaders != shader_key || z != last_z {
            if start != end {
                commands.spawn(UiShapeBatch {
                    range: start..end,
                    shader_key: current_batch_shaders,
                    pipeline: current_batch_pipeline,
                    z: FloatOrd(last_z),
                });
                start = end;
            }
            current_batch_shaders = shader_key;

            current_batch_pipeline = match smud_pipeline.shaders.0.get(&shader_key) {
                Some(_shader) => {
                    // todo pass the shader into specialize
                    let specialize_key = SmudPipelineKey {
                        mesh: mesh_key,
                        shader: shader_key,
                    };
                    pipelines.specialize(&mut pipeline_cache, &smud_pipeline, specialize_key)
                }
                None => CachedRenderPipelineId::INVALID,
            }
        }

        if current_batch_pipeline == CachedRenderPipelineId::INVALID {
            debug!("Shape not ready yet, skipping");
            continue; // skip shapes that are not ready yet
        }

        let color = extracted_shape.color.as_linear_rgba_f32();
        let params = extracted_shape.params.to_array();

        let position = position.into();

        let rotation_and_scale = extracted_shape
            .transform
            .affine()
            .transform_vector3(Vec3::X)
            .xy();

        let scale = rotation_and_scale.length();
        let rotation = (rotation_and_scale / scale).into();

        let vertex = ShapeVertex {
            position,
            color,
            params,
            rotation,
            scale,
            frame: extracted_shape.frame,
        };
        debug!("{vertex:?}");
        shape_meta.ui_vertices.push(vertex);
        last_z = z;
        end += 1;
    }

    // if start != end, there is one last batch to process
    if start != end {
        commands.spawn(UiShapeBatch {
            range: start..end,
            shader_key: current_batch_shaders,
            z: FloatOrd(last_z),
            pipeline: current_batch_pipeline,
        });
    }

    shape_meta
        .ui_vertices
        .write_buffer(&render_device, &render_queue);
}

fn queue_ui_shapes(
    transparent_draw_functions: Res<DrawFunctions<TransparentUi>>,
    ui_shape_batches: Query<(Entity, &UiShapeBatch)>,
    mut views: Query<&mut RenderPhase<TransparentUi>>,
) {
    // TODO: look at both the shape renderer and the
    // ui renderer and figure out which part to copy here!!!

    let draw_smud_ui_shape = transparent_draw_functions
        .read()
        // TODO: compare with ui draw command
        .get_id::<DrawSmudUiShape>()
        .unwrap();

    for mut transparent_phase in views.iter_mut() {
        for (entity, batch) in ui_shape_batches.iter() {
            // TODO: specializing seems to normally be done in queue. Move it here?
            let pipeline = batch.pipeline;
            transparent_phase.add(TransparentUi {
                draw_function: draw_smud_ui_shape,
                pipeline,
                entity,
                sort_key: batch.z,
            });
        }
    }
}

#[derive(Component, Eq, PartialEq, Clone)]
pub struct UiShapeBatch {
    range: Range<u32>,
    shader_key: (HandleId, HandleId),
    z: FloatOrd,
    pipeline: CachedRenderPipelineId,
}
