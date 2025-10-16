//! Provides `SmudNode` component for rendering SDF shapes in Bevy's UI

use bevy::{
    ecs::system::{
        SystemParamItem,
        lifetimeless::{Read, SRes},
    },
    math::{Affine2, Rect},
    prelude::*,
    render::{
        Extract, ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            BufferUsages, CachedPipelineState, ColorTargetState, ColorWrites, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RawBufferVec, RenderPipelineDescriptor, ShaderStages,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexAttribute,
            VertexFormat, VertexState, VertexStepMode, binding_types::uniform_buffer,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::{MainEntity, TemporaryRenderEntity},
        view::{ViewUniform, ViewUniformOffset, ViewUniforms},
    },
    ui::{ComputedNode, Node, UiGlobalTransform},
    ui_render::{TransparentUi, stack_z_offsets},
};
use bytemuck::{Pod, Zeroable};

use crate::{
    DEFAULT_FILL_HANDLE, FloatOrd, GeneratedShaders, VertexBufferLayout,
    shader_loading::VERTEX_SHADER_HANDLE,
};

/// Component for rendering SMUD shapes in UI.
///
/// This component requires `Node` and renders an SDF-based shape within the UI node bounds.
#[derive(Component, Reflect, Debug, Clone)]
#[require(Node)]
#[reflect(Component)]
pub struct SmudNode {
    /// The color of the shape
    pub color: Color,

    /// Shader containing a wgsl function for a signed distance field
    ///
    /// The shader needs to have the signature `fn sdf(input: smud::SdfInput) -> f32`.
    pub sdf: Handle<Shader>,

    /// Shader containing a wgsl function for the fill of the shape
    ///
    /// The shader needs to have the signature `fn fill(input: smud::FillInput) -> vec4<f32>`.
    pub fill: Handle<Shader>,

    /// Parameters to pass to shapes, for things such as width of a box
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SmudUiVertex {
    position: [f32; 3],
    color: [f32; 4],
    params: [f32; 4],
    rotation: [f32; 2],
    scale: f32,
    bounds: [f32; 2],
}

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

struct ExtractedSmudNode {
    main_entity: MainEntity,
    render_entity: Entity,
    stack_index: u32,
    transform: Affine2,
    /// Node bounds in local space
    rect: Rect,
    color: Color,
    params: Vec4,
    shader: Handle<Shader>,
}

/// Resource holding all extracted SmudNodes for the current frame
#[derive(Resource, Default)]
struct ExtractedSmudNodes {
    nodes: Vec<ExtractedSmudNode>,
}

// TODO: do some of this work in the main world instead, so we don't need to take a mutable
// reference to MainWorld.
fn generate_shaders(
    mut main_world: ResMut<MainWorld>,
    mut generated_shaders: ResMut<GeneratedShaders>,
) {
    main_world.resource_scope(|world, mut shaders: Mut<Assets<Shader>>| {
        let mut ui_nodes = world.query::<&SmudNode>();

        for node in ui_nodes.iter(world) {
            generated_shaders.try_generate(&node.sdf, &node.fill, &mut shaders);
        }
    });
}
/// Extract SmudNode components to render world
fn extract_smud_nodes(
    mut commands: Commands,
    mut extracted_nodes: ResMut<ExtractedSmudNodes>,
    generated_shaders: Res<GeneratedShaders>,
    smud_nodes: Extract<Query<(Entity, &SmudNode, &ComputedNode, &UiGlobalTransform)>>,
) {
    extracted_nodes.nodes.clear();

    for (entity, smud_node, computed_node, transform) in smud_nodes.iter() {
        let render_entity = commands.spawn(TemporaryRenderEntity).id();

        let Some(shader) = generated_shaders
            .0
            .get(&(smud_node.sdf.id(), smud_node.fill.id()))
            .cloned()
        else {
            // Shader not yet generated - skip this node for now
            continue;
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
            shader,
        });
    }
}

/// Pipeline key for specializing UI rendering based on shaders
#[derive(Clone, Hash, PartialEq, Eq)]
struct SmudUiPipelineKey {
    shader: Handle<Shader>,
}

/// Pipeline for rendering SMUD UI shapes
#[derive(Resource)]
struct SmudUiPipeline {
    view_layout: BindGroupLayout,
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

        Self { view_layout }
    }
}

impl SpecializedRenderPipeline for SmudUiPipeline {
    type Key = SmudUiPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Get the generated shader for this sdf+fill combination
        let shader = key.shader;

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

/// Prepare vertex buffers - generates vertices for each extracted node
fn prepare_smud_ui(
    mut smud_ui_meta: ResMut<SmudUiMeta>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_nodes: Res<ExtractedSmudNodes>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<SmudUiPipeline>,
) {
    // Create view bind group
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        smud_ui_meta.view_bind_group = Some(render_device.create_bind_group(
            "smud_ui_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));
    }

    smud_ui_meta.vertices.clear();

    // Generate one instance per node - vertex shader will use vertex_index to determine corners
    for node in &extracted_nodes.nodes {
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
    }

    smud_ui_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}

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
                shader: node.shader.clone(),
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

type DrawSmudUi = (SetItemPipeline, SetSmudUiViewBindGroup<0>, DrawSmudUiBatch);

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

struct DrawSmudUiBatch;

impl RenderCommand<TransparentUi> for DrawSmudUiBatch {
    type Param = SRes<SmudUiMeta>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &TransparentUi,
        _view: (),
        _entity: Option<()>,
        smud_ui_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let smud_ui_meta = smud_ui_meta.into_inner();
        let Some(vertices) = smud_ui_meta.vertices.buffer() else {
            return RenderCommandResult::Failure("no vertex buffer");
        };

        // Get the index of this specific UI node from the phase item
        let node_index = item.index as u32;

        pass.set_vertex_buffer(0, vertices.slice(..));
        // Draw 4 vertices for THIS specific instance only
        // Each instance uses 4 vertices in a triangle strip
        pass.draw(0..4, node_index..(node_index + 1));
        RenderCommandResult::Success
    }
}

/// Plugin for rendering smud shapes in bevy_ui
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
                    (generate_shaders, extract_smud_nodes.after(generate_shaders)),
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
