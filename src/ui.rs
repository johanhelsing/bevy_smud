//! Provides `SmudNode` component for rendering SDF shapes in Bevy's UI

use bevy::{
    asset::uuid_handle,
    ecs::system::{
        lifetimeless::{Read, SRes},
        SystemParamItem,
    },
    math::{Affine2, Rect},
    prelude::*,
    render::{
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, BlendState, BufferUsages, CachedPipelineState,
            ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology, RawBufferVec,
            RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexFormat, VertexState,
            VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::{MainEntity, TemporaryRenderEntity},
        view::{ViewUniform, ViewUniformOffset, ViewUniforms},
        Extract, ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems,
    },
    ui::{ComputedNode, Node, UiGlobalTransform},
    ui_render::{stack_z_offsets, TransparentUi},
};
use bytemuck::{Pod, Zeroable};
use std::ops::Range;

use crate::{
    shader_loading::VERTEX_SHADER_HANDLE, FloatOrd, GeneratedShaders, SmudPipeline,
    VertexBufferLayout, DEFAULT_FILL_HANDLE,
};

const TEST_UI_SHADER_HANDLE: Handle<Shader> = uuid_handle!("f1e2d3c4-b5a6-9788-0011-223344556677");

/// Component for rendering SMUD shapes in UI.
///
/// This component requires `Node` and renders an SDF-based shape within the UI node bounds.
#[derive(Component, Reflect, Debug, Clone)]
#[require(Node)]
#[reflect(Component)]
pub struct SmudNode {
    /// The color of the shape
    pub color: Color,

    /// SDF shader handle
    pub sdf: Handle<Shader>,

    /// Fill shader handle
    pub fill: Handle<Shader>,

    /// Parameters passed to SDF shader (e.g., radius, corner radius, etc.)
    pub params: Vec4,
}

impl Default for SmudNode {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            sdf: Handle::default(),
            fill: DEFAULT_FILL_HANDLE,
            params: Vec4::ZERO,
        }
    }
}

/// Vertex data for instanced UI rendering
/// Matches the regular Vertex structure from vertex.wgsl
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SmudUiVertex {
    /// World position (center of the UI node)
    position: [f32; 3],
    /// Color
    color: [f32; 4],
    /// Parameters passed to SDF shader
    params: [f32; 4],
    /// Rotation (cos, sin)
    rotation: [f32; 2],
    /// Scale
    scale: f32,
    /// Bounds (width, height)
    bounds: [f32; 2],
}

#[derive(Component)]
struct SmudUiBatch {
    range: Range<u32>,
}

/// Resource holding vertex buffers for UI shapes
#[derive(Resource)]
struct SmudUiMeta {
    vertices: RawBufferVec<SmudUiVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for SmudUiMeta {
    fn default() -> Self {
        Self {
            vertices: RawBufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

/// Extracted SmudNode data in render world
struct ExtractedSmudNode {
    /// Main world entity
    main_entity: MainEntity,
    /// Render world entity
    render_entity: Entity,
    /// Stack index for z-ordering
    stack_index: u32,
    /// Transform to world space
    transform: Affine2,
    /// Node bounds in local space
    rect: Rect,
    /// Color
    color: Color,
    /// SDF params
    params: Vec4,
    /// SDF shader handle
    sdf_shader: Handle<Shader>,
    /// Fill shader handle
    fill_shader: Handle<Shader>,
    /// Pipeline key for batching
    pipeline_key: SmudUiPipelineKey,
}

/// Resource holding all extracted SmudNodes for the current frame
#[derive(Resource, Default)]
struct ExtractedSmudNodes {
    nodes: Vec<ExtractedSmudNode>,
}

// TODO: do some of this work in the main world instead, so we don't need to take a mutable
// reference to MainWorld.
fn generate_shaders(mut main_world: ResMut<MainWorld>, mut pipeline: ResMut<SmudPipeline>) {
    main_world.resource_scope(|world, mut shaders: Mut<Assets<Shader>>| {
        let mut ui_nodes = world.query::<&SmudNode>();

        for node in ui_nodes.iter(world) {
            pipeline
                .shaders
                .maybe_generate(&node.sdf, &node.fill, &mut shaders);
        }
    });
}
/// Extract SmudNode components to render world
fn extract_smud_nodes(
    mut commands: Commands,
    mut extracted_nodes: ResMut<ExtractedSmudNodes>,
    smud_nodes: Extract<Query<(Entity, &SmudNode, &ComputedNode, &UiGlobalTransform)>>,
) {
    extracted_nodes.nodes.clear();

    for (entity, smud_node, computed_node, transform) in smud_nodes.iter() {
        let render_entity = commands.spawn(TemporaryRenderEntity).id();

        let pipeline_key = SmudUiPipelineKey {
            shader: (smud_node.sdf.id(), smud_node.fill.id()),
        };

        extracted_nodes.nodes.push(ExtractedSmudNode {
            main_entity: entity.into(),
            render_entity,
            stack_index: computed_node.stack_index,
            transform: transform.into(),
            rect: Rect {
                min: Vec2::ZERO,
                max: computed_node.size,
            },
            color: smud_node.color,
            params: smud_node.params,
            sdf_shader: smud_node.sdf.clone(),
            fill_shader: smud_node.fill.clone(),
            pipeline_key,
        });
    }
}

/// Sync composed shaders from SmudPipeline to SmudUiPipeline
fn clone_shaders_to_pipeline(
    main_pipeline: Res<SmudPipeline>,
    mut ui_pipeline: ResMut<SmudUiPipeline>,
) {
    // TODO: can we get rid of this cloning?
    ui_pipeline.shaders.0.clone_from(&main_pipeline.shaders.0);
}

/// Pipeline key for specializing UI rendering based on shaders
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct SmudUiPipelineKey {
    /// Tuple of (SDF shader, fill shader)
    shader: (AssetId<Shader>, AssetId<Shader>),
}

/// Pipeline for rendering SMUD UI shapes
#[derive(Resource)]
struct SmudUiPipeline {
    view_layout: BindGroupLayout,
    shaders: GeneratedShaders,
}

impl FromWorld for SmudUiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            "smud_ui_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        Self {
            view_layout,
            shaders: Default::default(),
        }
    }
}

impl SpecializedRenderPipeline for SmudUiPipeline {
    type Key = SmudUiPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Get the composed shader for this SDF+fill combination
        // If not found, fall back to the test shader (shouldn't happen in practice)
        let shader = self.shaders.0.get(&key.shader).cloned().unwrap_or_else(|| {
            bevy::log::warn!(
                "UI shader not found for key {:?}, using fallback",
                key.shader
            );
            TEST_UI_SHADER_HANDLE
        });

        RenderPipelineDescriptor {
            label: Some("smud_ui_pipeline".into()),
            layout: vec![self.view_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: VERTEX_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout {
                    array_stride: std::mem::size_of::<SmudUiVertex>() as u64,
                    step_mode: VertexStepMode::Instance, // One instance per UI node
                    attributes: vec![
                        // position
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        // color
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 12,
                            shader_location: 1,
                        },
                        // params
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 28,
                            shader_location: 2,
                        },
                        // rotation
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 44,
                            shader_location: 3,
                        },
                        // scale
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 52,
                            shader_location: 4,
                        },
                        // bounds
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 56,
                            shader_location: 5,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                shader,
                shader_defs: vec![],
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb, // UI render target format
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

/// Prepare vertex buffers - generates vertices for each extracted node and creates batches
fn prepare_smud_ui(
    mut commands: Commands,
    mut smud_ui_meta: ResMut<SmudUiMeta>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_nodes: Res<ExtractedSmudNodes>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<SmudUiPipeline>,
    mut phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    mut previous_len: Local<usize>,
) {
    // Create view bind group
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        smud_ui_meta.view_bind_group = Some(render_device.create_bind_group(
            "smud_ui_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));
    }

    let mut batches: Vec<(Entity, SmudUiBatch)> = Vec::with_capacity(*previous_len);

    smud_ui_meta.vertices.clear();

    // Track the current instance index
    let mut instance_index = 0u32;

    // Process all UI phases
    for ui_phase in phases.values_mut() {
        let mut batch_start_instance = instance_index;
        let mut batch_pipeline_key: Option<SmudUiPipelineKey> = None;
        let mut batch_start_item_index = 0;

        for item_index in 0..ui_phase.items.len() {
            let item = &ui_phase.items[item_index];

            // Find the corresponding extracted node
            let Some(node) = extracted_nodes
                .nodes
                .get(item.index)
                .filter(|n| item.entity() == n.render_entity)
            else {
                // Reset batch if we can't find the node
                batch_pipeline_key = None;
                continue;
            };

            // Check if we need to start a new batch
            let should_start_new_batch = match batch_pipeline_key {
                None => true,
                Some(current_key) => current_key != node.pipeline_key,
            };

            if should_start_new_batch {
                // Finalize previous batch if it exists
                if let Some(_) = batch_pipeline_key
                    && batch_start_instance < instance_index {
                        // Add the batch for the previous group
                        let batch_entity = ui_phase.items[batch_start_item_index].entity();
                        batches.push((
                            batch_entity,
                            SmudUiBatch {
                                range: batch_start_instance..instance_index,
                            },
                        ));
                    }

                // Start new batch
                batch_start_instance = instance_index;
                batch_start_item_index = item_index;
                batch_pipeline_key = Some(node.pipeline_key);
            }

            // Generate vertex data for this node
            let rect_size = node.rect.size();

            // Extract transform components from Affine2
            let position = node.transform.translation;

            // Extract rotation and scale from the matrix
            let x_axis = node.transform.matrix2.x_axis;
            let y_axis = node.transform.matrix2.y_axis;

            // Scale is the length of the axis vectors
            let scale_x = x_axis.length();
            let scale_y = y_axis.length();
            let scale = (scale_x + scale_y) / 2.0; // Average scale (assuming uniform)

            // Rotation is the direction of the x-axis (normalized)
            let rotation = if scale_x > 0.0 {
                let normalized = x_axis / scale_x;
                [normalized.x, normalized.y] // [cos, sin]
            } else {
                [1.0, 0.0] // No rotation
            };

            smud_ui_meta.vertices.push(SmudUiVertex {
                position: [position.x, position.y, 0.0],
                color: node.color.to_linear().to_f32_array(),
                params: node.params.to_array(),
                rotation,
                scale,
                bounds: [rect_size.x / 2.0, rect_size.y / 2.0],
            });

            instance_index += 1;
        }

        // Now update batch ranges in a second pass
        for item_index in 0..ui_phase.items.len() {
            let item = &mut ui_phase.items[item_index];
            *item.batch_range_mut() = item_index as u32..item_index as u32 + 1;
        }

        // Finalize the last batch if it exists
        if batch_pipeline_key.is_some() && batch_start_instance < instance_index
            && let Some(last_item) = ui_phase.items.last() {
                batches.push((
                    last_item.entity(),
                    SmudUiBatch {
                        range: batch_start_instance..instance_index,
                    },
                ));
            }
    }

    smud_ui_meta
        .vertices
        .write_buffer(&render_device, &render_queue);

    *previous_len = batches.len();
    commands.insert_batch(batches);
}

/// Queue system - adds SmudNode items to the TransparentUi render phase
fn queue_smud_ui(
    draw_functions: Res<DrawFunctions<TransparentUi>>,
    pipeline: Res<SmudUiPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudUiPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    extracted_nodes: Res<ExtractedSmudNodes>,
) {
    let draw_function = draw_functions.read().id::<DrawSmudUi>();

    // For each view that has a TransparentUi phase
    for (_view_key, transparent_phase) in transparent_render_phases.iter_mut() {
        // Add each extracted SmudNode to the render phase
        for (index, node) in extracted_nodes.nodes.iter().enumerate() {
            // Create pipeline key for this shader combination
            let key = SmudUiPipelineKey {
                shader: (node.sdf_shader.id(), node.fill_shader.id()),
            };

            // Specialize the pipeline for this shader combination
            let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, key);

            // Check if pipeline is ready - if not, skip this node
            if !matches!(
                pipeline_cache.get_render_pipeline_state(pipeline_id),
                CachedPipelineState::Ok(_)
            ) {
                continue;
            }

            // Use stack_index with an offset to control z-ordering
            // We use a value slightly after MATERIAL (0.05) so SmudNodes render in proper layer order
            let sort_key = FloatOrd(node.stack_index as f32 + stack_z_offsets::MATERIAL + 0.01);

            transparent_phase.add(TransparentUi {
                entity: (node.render_entity, node.main_entity),
                draw_function,
                pipeline: pipeline_id,
                sort_key,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                index,
                indexed: false,
            });
        }
    }
}

/// Draw command for rendering SmudNodes - tuple of render commands
type DrawSmudUi = (SetItemPipeline, SetSmudUiViewBindGroup<0>, DrawSmudUiBatch);

/// Set the view bind group
struct SetSmudUiViewBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetSmudUiViewBindGroup<I> {
    type Param = SRes<SmudUiMeta>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: &'w ViewUniformOffset,
        _entity: Option<()>,
        smud_ui_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(view_bind_group) = smud_ui_meta.into_inner().view_bind_group.as_ref() else {
            return RenderCommandResult::Failure("view_bind_group not available");
        };
        pass.set_bind_group(I, view_bind_group, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

/// Actual draw implementation
struct DrawSmudUiBatch;

impl RenderCommand<TransparentUi> for DrawSmudUiBatch {
    type Param = SRes<SmudUiMeta>;
    type ViewQuery = ();
    type ItemQuery = Read<SmudUiBatch>;

    fn render<'w>(
        _item: &TransparentUi,
        _view: (),
        batch: Option<&'w SmudUiBatch>,
        smud_ui_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(batch) = batch else {
            return RenderCommandResult::Skip;
        };

        let smud_ui_meta = smud_ui_meta.into_inner();
        let Some(vertices) = smud_ui_meta.vertices.buffer() else {
            return RenderCommandResult::Failure("no vertex buffer");
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        // Draw 4 vertices for all instances in this batch
        // Each instance uses 4 vertices in a triangle strip
        pass.draw(0..4, batch.range.clone());
        RenderCommandResult::Success
    }
}

/// Plugin for rendering SMUD shapes in UI
pub(crate) struct UiShapePlugin;

impl Plugin for UiShapePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmudNode>();
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<TransparentUi, DrawSmudUi>()
                .init_resource::<SmudUiMeta>()
                .init_resource::<SmudUiPipeline>()
                .init_resource::<ExtractedSmudNodes>()
                .init_resource::<SpecializedRenderPipelines<SmudUiPipeline>>()
                .add_systems(
                    ExtractSchedule,
                    (
                        generate_shaders,
                        extract_smud_nodes,
                        clone_shaders_to_pipeline.after(generate_shaders),
                    ),
                )
                .add_systems(
                    Render,
                    (
                        queue_smud_ui.in_set(RenderSystems::Queue),
                        prepare_smud_ui.in_set(RenderSystems::PrepareResources),
                    ),
                );
        }
    }
}
