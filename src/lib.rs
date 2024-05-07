#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(clippy::too_many_arguments)]

use std::ops::Range;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        entity::EntityHashMap,
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::Vec3Swizzles,
    prelude::*,
    render::{
        globals::{GlobalsBuffer, GlobalsUniform},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, BufferUsages, BufferVec, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, ShaderImport, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{
            ExtractedView, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms,
            VisibleEntities,
        },
        Extract, MainWorld, Render, RenderApp, RenderSet,
    },
    utils::{FloatOrd, HashMap},
};
use bytemuck::{Pod, Zeroable};
use fixedbitset::FixedBitSet;
use shader_loading::*;
// use ui::UiShapePlugin;

pub use bundle::ShapeBundle;
pub use components::*;
pub use shader_loading::{DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE};

use crate::util::generate_shader_id;

mod bundle;
mod components;
mod sdf_assets;
mod shader_loading;
mod util;
// mod ui;

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_smud::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        sdf_assets::SdfAssets,
        Frame,
        ShapeBundle,
        SmudPlugin,
        SmudShape,
        // UiShapeBundle,
        DEFAULT_FILL_HANDLE,
        SIMPLE_FILL_HANDLE,
    };
}

#[derive(Default)]
/// Main plugin for enabling rendering of Sdf shapes
pub struct SmudPlugin;

impl Plugin for SmudPlugin {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugins(ShaderLoadingPlugin);
        // app.add_plugins(UiShapePlugin);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape>()
                .init_resource::<ExtractedShapes>()
                .init_resource::<ShapeMeta>()
                .init_resource::<SpecializedRenderPipelines<SmudPipeline>>()
                .add_systems(ExtractSchedule, (extract_shapes, extract_sdf_shaders))
                .add_systems(
                    Render,
                    (
                        queue_shapes.in_set(RenderSet::Queue),
                        prepare_shapes.in_set(RenderSet::PrepareBindGroups),
                    ),
                );
        }

        app.register_type::<SmudShape>();
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SmudPipeline>();
    }
}

type DrawSmudShape = (SetItemPipeline, SetShapeViewBindGroup<0>, DrawShapeBatch);

struct SetShapeViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetShapeViewBindGroup<I> {
    type Param = SRes<ShapeMeta>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            shape_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}

struct DrawShapeBatch;
impl<P: PhaseItem> RenderCommand<P> for DrawShapeBatch {
    type Param = SRes<ShapeMeta>;
    type ViewQuery = ();
    type ItemQuery = Read<ShapeBatch>;

    fn render<'w>(
        _item: &P,
        _view: (),
        batch: Option<ROQueryItem<'w, Self::ItemQuery>>,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let shape_meta = shape_meta.into_inner();
        pass.set_vertex_buffer(0, shape_meta.vertices.buffer().unwrap().slice(..));
        if let Some(batch) = batch {
            pass.draw(0..4, batch.range.clone());
        }
        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct SmudPipeline {
    view_layout: BindGroupLayout,
    shaders: ShapeShaders,
}

impl FromWorld for SmudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let view_layout = render_device.create_bind_group_layout(
            Some("shape_view_layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
            ],
        );

        Self {
            view_layout,
            shaders: default(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SmudPipelineKey {
    mesh: PipelineKey,
    shader: (AssetId<Shader>, AssetId<Shader>),
    hdr: bool,
}

impl SpecializedRenderPipeline for SmudPipeline {
    type Key = SmudPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader = self.shaders.0.get(&key.shader).unwrap();
        debug!("specializing for {shader:?}");

        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // (GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
            // Frame
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4) * 4,
                shader_location: 4,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: (4 + 1) * 4,
                shader_location: 0,
            },
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: (4 + 1 + 3) * 4,
                shader_location: 2,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4 + 1 + 3 + 2) * 4,
                shader_location: 3,
            },
        ];
        // This is the sum of the size of the attributes above
        let vertex_array_stride = (4 + 1 + 3 + 2 + 1) * 4;

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: VERTEX_SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader.clone_weak(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                // Bind group 0 is the view uniform
                self.view_layout.clone(),
            ],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false, // What is this?
                polygon_mode: PolygonMode::Fill,
                conservative: false, // What is this?
                topology: key.mesh.primitive_topology(),
                strip_index_format: None, // TODO: what does this do?
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.mesh.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("bevy_smud_pipeline".into()),
            push_constant_ranges: Vec::new(),
        }
    }
}

#[derive(Default)]
struct ShapeShaders(HashMap<(AssetId<Shader>, AssetId<Shader>), Handle<Shader>>);

// TODO: do some of this work in the main world instead, so we don't need to take a mutable
// reference to MainWorld.
fn extract_sdf_shaders(mut main_world: ResMut<MainWorld>, mut pipeline: ResMut<SmudPipeline>) {
    main_world.resource_scope(|world, mut shaders: Mut<Assets<Shader>>| {
        let mut shapes = world.query::<&SmudShape>();

        for shape in shapes.iter(world) {
            let shader_key = (shape.sdf.id(), shape.fill.id());
            if pipeline.shaders.0.contains_key(&shader_key) {
                continue;
            }

            // todo use asset events instead?
            let sdf_import_path = match shaders.get_mut(&shape.sdf.clone()) {
                Some(shader) => match shader.import_path() {
                    ShaderImport::Custom(p) => p.to_owned(),
                    _ => {
                        let id = generate_shader_id();
                        let path = format!("smud::generated::{id}");
                        shader.set_import_path(&path);
                        path
                    }
                },
                None => {
                    debug!("Waiting for sdf to load");
                    continue;
                }
            };

            let fill_import_path = match shaders.get_mut(&shape.fill.clone()) {
                Some(shader) => match shader.import_path() {
                    ShaderImport::Custom(p) => p.to_owned(),
                    _ => {
                        let id = generate_shader_id();
                        let path = format!("smud::generated::{id}");
                        shader.set_import_path(&path);
                        path
                    }
                },
                None => {
                    debug!("Waiting for fill to load");
                    continue;
                }
            };

            debug!("Generating shader");
            let generated_shader = Shader::from_wgsl(
                format!(
                    r#"
#import bevy_render::globals::Globals
@group(0) @binding(1)
var<uniform> globals: Globals;
#import {sdf_import_path} as sdf
#import {fill_import_path} as fill

struct FragmentInput {{
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
}};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {{
    let d = sdf::sdf(in.pos);
    return fill::fill(d, in.color);
}}
"#
                ),
                format!("smud::generated::{shader_key:?}"),
            );

            // todo does this work, or is it too late?
            let generated_shader_handle = shaders.add(generated_shader);

            pipeline
                .shaders
                .0
                .insert(shader_key, generated_shader_handle);
        }
    });
}

#[derive(Component, Clone, Debug)]
struct ExtractedShape {
    color: Color,
    frame: f32,
    sdf_shader: Handle<Shader>,
    fill_shader: Handle<Shader>,
    transform: GlobalTransform,
}

#[derive(Resource, Default, Debug)]
struct ExtractedShapes {
    shapes: EntityHashMap<ExtractedShape>,
}

fn extract_shapes(
    mut extracted_shapes: ResMut<ExtractedShapes>,
    shape_query: Extract<Query<(Entity, &ViewVisibility, &SmudShape, &GlobalTransform)>>,
) {
    extracted_shapes.shapes.clear();

    for (entity, view_visibility, shape, transform) in shape_query.iter() {
        if !view_visibility.get() {
            continue;
        }

        let Frame::Quad(frame) = shape.frame;

        extracted_shapes.shapes.insert(
            entity,
            ExtractedShape {
                color: shape.color,
                transform: *transform,
                sdf_shader: shape.sdf.clone_weak(),
                fill_shader: shape.fill.clone_weak(),
                frame,
            },
        );
    }
}

// fork of Mesh2DPipelineKey (in order to remove bevy_sprite dependency)
// todo: merge with SmudPipelineKey?
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    struct PipelineKey: u32 {
        const MSAA_RESERVED_BITS                = Self::MSAA_MASK_BITS << Self::MSAA_SHIFT_BITS;
        const PRIMITIVE_TOPOLOGY_RESERVED_BITS  = Self::PRIMITIVE_TOPOLOGY_MASK_BITS << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
    }
}

impl PipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();
    const PRIMITIVE_TOPOLOGY_MASK_BITS: u32 = 0b111;
    const PRIMITIVE_TOPOLOGY_SHIFT_BITS: u32 = Self::MSAA_SHIFT_BITS - 3;

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits =
            (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Self::from_bits(msaa_bits).unwrap()
    }

    pub fn msaa_samples(&self) -> u32 {
        1 << ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS)
    }

    pub fn from_primitive_topology(primitive_topology: PrimitiveTopology) -> Self {
        let primitive_topology_bits = ((primitive_topology as u32)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS)
            << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
        Self::from_bits(primitive_topology_bits).unwrap()
    }

    pub fn primitive_topology(&self) -> PrimitiveTopology {
        let primitive_topology_bits = (self.bits() >> Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS;
        match primitive_topology_bits {
            x if x == PrimitiveTopology::PointList as u32 => PrimitiveTopology::PointList,
            x if x == PrimitiveTopology::LineList as u32 => PrimitiveTopology::LineList,
            x if x == PrimitiveTopology::LineStrip as u32 => PrimitiveTopology::LineStrip,
            x if x == PrimitiveTopology::TriangleList as u32 => PrimitiveTopology::TriangleList,
            x if x == PrimitiveTopology::TriangleStrip as u32 => PrimitiveTopology::TriangleStrip,
            _ => PrimitiveTopology::default(),
        }
    }
}

fn queue_shapes(
    mut view_entities: Local<FixedBitSet>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    smud_pipeline: Res<SmudPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline>>,
    pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    extracted_shapes: ResMut<ExtractedShapes>,
    mut views: Query<(
        &mut RenderPhase<Transparent2d>,
        &VisibleEntities,
        &ExtractedView,
    )>,
    // ?
) {
    let draw_smud_shape_function = draw_functions.read().get_id::<DrawSmudShape>().unwrap();

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, visible_entities, view) in &mut views {
        // todo: bevy_sprite does some hdr stuff, should we?
        // let mut view_key = SpritePipelineKey::from_hdr(view.hdr) | msaa_key;

        let mesh_key = PipelineKey::from_msaa_samples(msaa.samples())
            | PipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        view_entities.clear();
        view_entities.extend(visible_entities.entities.iter().map(|e| e.index() as usize));

        transparent_phase
            .items
            .reserve(extracted_shapes.shapes.len());

        for (entity, extracted_shape) in extracted_shapes.shapes.iter() {
            let shader = (
                extracted_shape.sdf_shader.id(),
                extracted_shape.fill_shader.id(),
            );

            let mut pipeline = CachedRenderPipelineId::INVALID;

            if let Some(_shader) = smud_pipeline.shaders.0.get(&shader) {
                // todo pass the shader into specialize
                let specialize_key = SmudPipelineKey {
                    mesh: mesh_key,
                    shader,
                    hdr: view.hdr,
                };
                pipeline = pipelines.specialize(&pipeline_cache, &smud_pipeline, specialize_key);
            }

            if pipeline == CachedRenderPipelineId::INVALID {
                debug!("Shape not ready yet, skipping");
                continue; // skip shapes that are not ready yet
            }

            // These items will be sorted by depth with other phase items
            let sort_key = FloatOrd(extracted_shape.transform.translation().z);

            // Add the item to the render phase
            transparent_phase.add(Transparent2d {
                draw_function: draw_smud_shape_function,
                pipeline,
                entity: *entity,
                sort_key,
                // batch_range and dynamic_offset will be calculated in prepare_shapes
                batch_range: 0..0,
                dynamic_offset: None,
            });
        }
    }
}

fn prepare_shapes(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut shape_meta: ResMut<ShapeMeta>,
    view_uniforms: Res<ViewUniforms>,
    smud_pipeline: Res<SmudPipeline>,
    extracted_shapes: Res<ExtractedShapes>,
    mut phases: Query<&mut RenderPhase<Transparent2d>>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let globals = globals_buffer.buffer.binding().unwrap(); // todo if-let

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let mut batches: Vec<(Entity, ShapeBatch)> = Vec::with_capacity(*previous_len);

        // Clear the vertex buffer
        shape_meta.vertices.clear();

        shape_meta.view_bind_group = Some(render_device.create_bind_group(
            "smud_shape_view_bind_group",
            &smud_pipeline.view_layout,
            &BindGroupEntries::sequential((view_binding, globals.clone())),
        ));

        // Vertex buffer index
        let mut index = 0;

        for mut transparent_phase in &mut phases {
            let mut batch_item_index = 0;
            // let mut batch_image_size = Vec2::ZERO;
            // let mut batch_image_handle = AssetId::invalid();
            let mut batch_shader_handles = (AssetId::invalid(), AssetId::invalid());

            // Iterate through the phase items and detect when successive shapes that can be batched.
            // Spawn an entity with a `ShapeBatch` component for each possible batch.
            // Compatible items share the same entity.
            for item_index in 0..transparent_phase.items.len() {
                let item = &transparent_phase.items[item_index];
                let Some(extracted_shape) = extracted_shapes.shapes.get(&item.entity) else {
                    // If there is a phase item that is not a shape, then we must start a new
                    // batch to draw the other phase item(s) and to respect draw order. This can be
                    // done by invalidating the batch_shader_handles
                    batch_shader_handles = (AssetId::invalid(), AssetId::invalid());
                    continue;
                };

                let shader_handles = (
                    extracted_shape.sdf_shader.id(),
                    extracted_shape.fill_shader.id(),
                );

                let batch_shader_changed = batch_shader_handles != shader_handles;

                let color = extracted_shape.color.as_linear_rgba_f32();

                let position = extracted_shape.transform.translation();
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
                    rotation,
                    scale,
                    frame: extracted_shape.frame,
                };
                shape_meta.vertices.push(vertex);

                if batch_shader_changed {
                    batch_item_index = item_index;

                    batches.push((
                        item.entity,
                        ShapeBatch {
                            shader: shader_handles,
                            range: index..index,
                        },
                    ));
                }

                transparent_phase.items[batch_item_index]
                    .batch_range_mut()
                    .end += 1;

                batches.last_mut().unwrap().1.range.end += 1;
                index += 1;
            }
        }

        shape_meta
            .vertices
            .write_buffer(&render_device, &render_queue);

        *previous_len = batches.len();
        commands.insert_or_spawn_batch(batches);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ShapeVertex {
    pub color: [f32; 4],
    pub frame: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
}

#[derive(Resource)]
pub(crate) struct ShapeMeta {
    vertices: BufferVec<ShapeVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for ShapeMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

#[derive(Component, Eq, PartialEq, Clone)]
pub(crate) struct ShapeBatch {
    shader: (AssetId<Shader>, AssetId<Shader>),
    range: Range<u32>,
}
