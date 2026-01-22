//! Provides `UiShape` component for rendering SDF shapes in Bevy's UI

use bevy::{
    ecs::system::{
        SystemParamItem,
        lifetimeless::{Read, SRes},
    },
    math::{Affine2, Rect},
    prelude::*,
    render::{
        Extract, ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems,
        globals::{GlobalsBuffer, GlobalsUniform},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            BlendComponent, BlendFactor, BlendOperation, BlendState, BufferUsages,
            CachedPipelineState, ColorTargetState, ColorWrites, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RawBufferVec, RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexFormat, VertexState,
            VertexStepMode, binding_types::uniform_buffer,
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
    BlendMode, FloatOrd, GeneratedShaders, SIMPLE_FILL_HANDLE, VertexBufferLayout,
    shader_loading::VERTEX_SHADER_HANDLE,
};

/// Component for rendering shapes in UI.
///
/// This component requires `Node` and renders an SDF-based shape within the UI node bounds.
#[derive(Component, Reflect, Debug, Clone)]
#[require(Node)]
#[reflect(Component)]
pub struct UiShape {
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

    /// Blend mode for the shape
    pub blend_mode: BlendMode,

    /// Extra padding to add to the bounds when rendering the shape
    pub extra_bounds: f32,
}

impl Default for UiShape {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            sdf: Handle::default(),
            fill: SIMPLE_FILL_HANDLE.clone(),
            params: Vec4::ZERO,
            blend_mode: BlendMode::default(),
            extra_bounds: 0.0,
        }
    }
}

impl UiShape {
    /// Set the blend mode for this shape (builder pattern)
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UiShapeVertex {
    position: [f32; 3],
    color: [f32; 4],
    params: [f32; 4],
    rotation: [f32; 2],
    scale: f32,
    bounds: [f32; 4],
}

#[derive(Resource)]
struct UiShapeMeta {
    vertices: RawBufferVec<UiShapeVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for UiShapeMeta {
    fn default() -> Self {
        Self {
            vertices: RawBufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

struct ExtractedUiShape {
    main_entity: MainEntity,
    render_entity: Entity,
    stack_index: u32,
    transform: Affine2,
    /// Node bounds in local space
    rect: Rect,
    extra_bounds: f32,
    color: Color,
    params: Vec4,
    shader: Handle<Shader>,
    blend_mode: BlendMode,
}

#[derive(Resource, Default)]
struct ExtractedUiShapes {
    nodes: Vec<ExtractedUiShape>,
}

// TODO: do some of this work in the main world instead, so we don't need to take a mutable
// reference to MainWorld.
fn generate_shaders(
    mut main_world: ResMut<MainWorld>,
    mut generated_shaders: ResMut<GeneratedShaders>,
) {
    main_world.resource_scope(|world, mut shaders: Mut<Assets<Shader>>| {
        for node in world.query::<&UiShape>().iter(world) {
            generated_shaders.try_generate(&node.sdf, &node.fill, &mut shaders);
        }
    });
}

/// Extract UiShape components to render world
fn extract_ui_shapes(
    mut commands: Commands,
    mut extracted_nodes: ResMut<ExtractedUiShapes>,
    generated_shaders: Res<GeneratedShaders>,
    ui_shapes: Extract<Query<(Entity, &UiShape, &ComputedNode, &UiGlobalTransform)>>,
) {
    extracted_nodes.nodes.clear();

    for (entity, ui_shape, computed_node, transform) in ui_shapes.iter() {
        let render_entity = commands.spawn(TemporaryRenderEntity).id();

        let Some(shader) = generated_shaders
            .0
            .get(&(ui_shape.sdf.id(), ui_shape.fill.id()))
            .cloned()
        else {
            // Shader not yet generated - skip this node for now
            continue;
        };

        extracted_nodes.nodes.push(ExtractedUiShape {
            main_entity: entity.into(),
            render_entity,
            stack_index: computed_node.stack_index,
            transform: transform.into(),
            rect: Rect {
                min: Vec2::ZERO,
                max: computed_node.size,
            },
            extra_bounds: ui_shape.extra_bounds,
            color: ui_shape.color,
            params: ui_shape.params,
            shader,
            blend_mode: ui_shape.blend_mode,
        });
    }
}

/// Pipeline key for specializing UI rendering based on shaders
#[derive(Clone, Hash, PartialEq, Eq)]
struct UiShapePipelineKey {
    shader: Handle<Shader>,
    blend_mode: BlendMode,
}

/// Pipeline for rendering shapes in UI.
#[derive(Resource)]
struct UiShapePipeline {
    view_layout: BindGroupLayoutDescriptor,
}

impl FromWorld for UiShapePipeline {
    fn from_world(_world: &mut World) -> Self {
        let entries = BindGroupLayoutEntries::with_indices(
            ShaderStages::VERTEX_FRAGMENT,
            (
                (0, uniform_buffer::<ViewUniform>(true)),
                (
                    1,
                    uniform_buffer::<GlobalsUniform>(false).visibility(ShaderStages::FRAGMENT),
                ),
            ),
        );

        let view_layout = BindGroupLayoutDescriptor::new("ui_shape_view_layout", &entries);

        Self { view_layout }
    }
}

impl SpecializedRenderPipeline for UiShapePipeline {
    type Key = UiShapePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Get the generated shader for this sdf+fill combination
        let shader = key.shader;

        RenderPipelineDescriptor {
            label: Some("ui_shape_pipeline".into()),
            layout: vec![self.view_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: VERTEX_SHADER_HANDLE,
                shader_defs: vec!["Y_DOWN".into()],
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiShapeVertex>() as u64,
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
                            format: VertexFormat::Float32x4,
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
                    blend: Some(match key.blend_mode {
                        BlendMode::Alpha => BlendState::ALPHA_BLENDING,
                        BlendMode::Additive => BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                        },
                    }),
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
fn prepare_ui_shapes(
    mut ui_shape_meta: ResMut<UiShapeMeta>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    extracted_nodes: Res<ExtractedUiShapes>,
    view_uniforms: Res<ViewUniforms>,
    globals_buffer: Res<GlobalsBuffer>,
    pipeline: Res<UiShapePipeline>,
    pipeline_cache: Res<PipelineCache>,
) {
    // Create view bind group
    if let (Some(view_binding), Some(globals)) = (
        view_uniforms.uniforms.binding(),
        globals_buffer.buffer.binding(),
    ) {
        let view_layout = pipeline_cache.get_bind_group_layout(&pipeline.view_layout);
        ui_shape_meta.view_bind_group = Some(render_device.create_bind_group(
            "ui_shape_view_bind_group",
            &view_layout,
            &BindGroupEntries::with_indices(((0, view_binding), (1, globals))),
        ));
    }

    ui_shape_meta.vertices.clear();

    // Generate one instance per node - vertex shader will use vertex_index to determine corners
    for node in &extracted_nodes.nodes {
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

        let bounds = node.rect.size() / 2.0;

        ui_shape_meta.vertices.push(UiShapeVertex {
            position: [position.x, position.y, 0.0],
            color: node.color.to_linear().to_f32_array(),
            params: node.params.to_array(),
            rotation,
            scale,
            bounds: [bounds.x, bounds.y, node.extra_bounds, node.extra_bounds],
        });
    }

    ui_shape_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}

fn queue_ui_shapes(
    draw_functions: Res<DrawFunctions<TransparentUi>>,
    pipeline: Res<UiShapePipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<UiShapePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
    extracted_nodes: Res<ExtractedUiShapes>,
) {
    let draw_function = draw_functions.read().id::<DrawUiShapes>();

    // For each view that has a TransparentUi phase
    for (_view_key, transparent_phase) in transparent_render_phases.iter_mut() {
        // Add each extracted UiShape to the render phase
        for (index, node) in extracted_nodes.nodes.iter().enumerate() {
            // Create pipeline key for this shader combination
            let key = UiShapePipelineKey {
                shader: node.shader.clone(),
                blend_mode: node.blend_mode,
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
            // We use a value slightly after MATERIAL (0.05) so UiShapes render in proper layer order
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

type DrawUiShapes = (
    SetItemPipeline,
    SetUiShapeViewBindGroup<0>,
    DrawUiShapeBatch,
);

struct SetUiShapeViewBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetUiShapeViewBindGroup<I> {
    type Param = SRes<UiShapeMeta>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: &'w ViewUniformOffset,
        _entity: Option<()>,
        ui_shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(view_bind_group) = ui_shape_meta.into_inner().view_bind_group.as_ref() else {
            return RenderCommandResult::Failure("view_bind_group not available");
        };
        pass.set_bind_group(I, view_bind_group, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

struct DrawUiShapeBatch;

impl RenderCommand<TransparentUi> for DrawUiShapeBatch {
    type Param = SRes<UiShapeMeta>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &TransparentUi,
        _view: (),
        _entity: Option<()>,
        ui_shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let ui_shape_meta = ui_shape_meta.into_inner();
        let Some(vertices) = ui_shape_meta.vertices.buffer() else {
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

/// Plugin for rendering shapes in bevy_ui
pub(crate) struct UiShapePlugin;

impl Plugin for UiShapePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiShape>();
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<TransparentUi, DrawUiShapes>()
                .init_resource::<UiShapeMeta>()
                .init_resource::<UiShapePipeline>()
                .init_resource::<ExtractedUiShapes>()
                .init_resource::<SpecializedRenderPipelines<UiShapePipeline>>()
                .add_systems(
                    ExtractSchedule,
                    (generate_shaders, extract_ui_shapes.after(generate_shaders)),
                )
                .add_systems(
                    Render,
                    (
                        queue_ui_shapes.in_set(RenderSystems::Queue),
                        prepare_ui_shapes.in_set(RenderSystems::PrepareResources),
                    ),
                );
        }
    }
}
