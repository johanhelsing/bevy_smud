#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(clippy::too_many_arguments)]

use std::ops::Range;

use bevy::{
    core_pipeline::{
        core_2d::{CORE_2D_DEPTH_FORMAT, Transparent2d},
        tonemapping::{
            DebandDither, Tonemapping, TonemappingLuts, get_lut_bind_group_layout_entries,
            get_lut_bindings,
        },
    },
    ecs::{
        query::ROQueryItem,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    math::{FloatOrd, Vec3Swizzles},
    mesh::VertexBufferLayout,
    platform::collections::HashMap,
    prelude::*,
    render::{
        Extract, MainWorld, Render, RenderApp, RenderSystems,
        globals::{GlobalsBuffer, GlobalsUniform},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendComponent,
            BlendFactor, BlendOperation, BlendState, BufferUsages, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
            Face, FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode,
            PrimitiveState, PrimitiveTopology, RawBufferVec, RenderPipelineDescriptor,
            ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines, StencilFaceState,
            StencilState, TextureFormat, VertexAttribute, VertexFormat, VertexState,
            VertexStepMode, binding_types::uniform_buffer,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::{MainEntity, RenderEntity},
        texture::{FallbackImage, GpuImage},
        view::{
            ExtractedView, RenderVisibleEntities, RetainedViewEntity, ViewTarget, ViewUniform,
            ViewUniformOffset, ViewUniforms,
        },
    },
    shader::{ShaderDefVal, ShaderImport},
};
use bytemuck::{Pod, Zeroable};
use fixedbitset::FixedBitSet;
use shader_loading::*;
// use ui::UiShapePlugin;

pub use components::*;
pub use shader_loading::{DEFAULT_FILL_HANDLE, RECTANGLE_SDF_HANDLE, SIMPLE_FILL_HANDLE};

use crate::util::generate_shader_id;

mod components;
#[cfg(feature = "bevy_picking")]
mod picking_backend;
pub mod sdf;
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
        BlendMode, DEFAULT_FILL_HANDLE, RECTANGLE_SDF_HANDLE, SIMPLE_FILL_HANDLE, SmudPlugin,
        SmudShape, sdf_assets::SdfAssets,
    };

    pub use bevy::math::primitives::Rectangle;

    #[cfg(feature = "bevy_picking")]
    pub use crate::picking_backend::{
        SmudPickingCamera, SmudPickingPlugin, SmudPickingSettings, SmudPickingShape,
    };

    pub use crate::sdf;
}

#[derive(Default)]
/// Main plugin for enabling rendering of Sdf shapes
pub struct SmudPlugin;

/// System set for shape rendering.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ShapeSystem {
    /// Extract shapes
    ExtractShapes,
    // ComputeSlices,
}

impl Plugin for SmudPlugin {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugins(ShaderLoadingPlugin);
        // app.add_plugins(UiShapePlugin);

        app.register_type::<SmudShape>();
        // TODO: calculate bounds?

        // TODO: picking
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpecializedRenderPipelines<SmudPipeline>>()
                .init_resource::<ShapeMeta>()
                .init_resource::<ExtractedShapes>()
                .add_render_command::<Transparent2d, DrawSmudShape>()
                .add_systems(
                    ExtractSchedule,
                    (
                        extract_shapes.in_set(ShapeSystem::ExtractShapes),
                        extract_sdf_shaders,
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ShapeBatches>()
            .init_resource::<SmudPipeline>()
            .add_systems(
                Render,
                (
                    queue_shapes.in_set(RenderSystems::Queue),
                    prepare_shape_view_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                    prepare_shapes.in_set(RenderSystems::PrepareBindGroups),
                ),
            );
    }
}

type DrawSmudShape = (SetItemPipeline, SetShapeViewBindGroup<0>, DrawShapeBatch);

struct SetShapeViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetShapeViewBindGroup<I> {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<ShapeViewBindGroup>);
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        (view_uniform, shape_view_bind_group): ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &shape_view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

struct DrawShapeBatch;
impl<P: PhaseItem> RenderCommand<P> for DrawShapeBatch {
    type Param = (SRes<ShapeMeta>, SRes<ShapeBatches>);
    type ViewQuery = Read<ExtractedView>;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        (shape_meta, batches): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let shape_meta = shape_meta.into_inner();
        pass.set_vertex_buffer(0, shape_meta.vertices.buffer().unwrap().slice(..));
        let Some(batch) = batches.get(&(view.retained_view_entity, item.main_entity())) else {
            return RenderCommandResult::Skip;
        };
        pass.draw(0..4, batch.range.clone());
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

        let tonemapping_lut_entries = get_lut_bind_group_layout_entries();
        let view_layout = render_device.create_bind_group_layout(
            Some("shape_view_layout"),
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    (0, uniform_buffer::<ViewUniform>(true)),
                    (
                        1,
                        uniform_buffer::<GlobalsUniform>(false).visibility(ShaderStages::FRAGMENT),
                    ),
                    (
                        2,
                        tonemapping_lut_entries[0].visibility(ShaderStages::FRAGMENT),
                    ),
                    (
                        3,
                        tonemapping_lut_entries[1].visibility(ShaderStages::FRAGMENT),
                    ),
                ),
            ),
        );

        Self {
            view_layout,
            shaders: default(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SmudPipelineKey {
    /// Mix of bevy_render Mesh2DPipelineKey and SpritePipelineKey
    mesh: PipelineKey,
    shader: (AssetId<Shader>, AssetId<Shader>),
    hdr: bool,
}

impl SpecializedRenderPipeline for SmudPipeline {
    type Key = SmudPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();

        if key.mesh.contains(PipelineKey::TONEMAP_IN_SHADER) {
            shader_defs.push("TONEMAP_IN_SHADER".into());
            shader_defs.push(ShaderDefVal::UInt(
                "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX".into(),
                2,
            ));
            shader_defs.push(ShaderDefVal::UInt(
                "TONEMAPPING_LUT_SAMPLER_BINDING_INDEX".into(),
                3,
            ));

            let method = key
                .mesh
                .intersection(PipelineKey::TONEMAP_METHOD_RESERVED_BITS);

            match method {
                PipelineKey::TONEMAP_METHOD_NONE => {
                    shader_defs.push("TONEMAP_METHOD_NONE".into());
                }
                PipelineKey::TONEMAP_METHOD_REINHARD => {
                    shader_defs.push("TONEMAP_METHOD_REINHARD".into());
                }
                PipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE => {
                    shader_defs.push("TONEMAP_METHOD_REINHARD_LUMINANCE".into());
                }
                PipelineKey::TONEMAP_METHOD_ACES_FITTED => {
                    shader_defs.push("TONEMAP_METHOD_ACES_FITTED".into());
                }
                PipelineKey::TONEMAP_METHOD_AGX => {
                    shader_defs.push("TONEMAP_METHOD_AGX".into());
                }
                PipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM => {
                    shader_defs.push("TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM".into());
                }
                PipelineKey::TONEMAP_METHOD_BLENDER_FILMIC => {
                    shader_defs.push("TONEMAP_METHOD_BLENDER_FILMIC".into());
                }
                PipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE => {
                    shader_defs.push("TONEMAP_METHOD_TONY_MC_MAPFACE".into());
                }
                _ => {}
            }
            // Debanding is tied to tonemapping in the shader, cannot run without it.
            if key.mesh.contains(PipelineKey::DEBAND_DITHER) {
                shader_defs.push("DEBAND_DITHER".into());
            }
        }

        let shader = self.shaders.0.get(&key.shader).unwrap();
        debug!("specializing for {shader:?}");
        debug!("shader_defs: {shader_defs:?}");

        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position, color and params
        let vertex_attributes = vec![
            // (GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
            // Bounds
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: (4) * 4,
                shader_location: 5,
            },
            // perf: Maybe it's possible to pack this more efficiently?
            // Params
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (4 + 2) * 4,
                shader_location: 2,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: (4 + 2 + 4) * 4,
                shader_location: 0,
            },
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: (4 + 2 + 4 + 3) * 4,
                shader_location: 3,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4 + 2 + 4 + 3 + 2) * 4,
                shader_location: 4,
            },
        ];
        // This is the sum of the size of the attributes above
        let vertex_array_stride = (4 + 2 + 4 + 3 + 2 + 1) * 4;

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: VERTEX_SHADER_HANDLE,
                entry_point: Some("vertex".into()),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                entry_point: Some("fragment".into()),
                shader_defs,
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: Some(if key.mesh.contains(PipelineKey::BLEND_ADDITIVE) {
                        BlendState {
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
                        }
                    } else {
                        BlendState::ALPHA_BLENDING
                    }),
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
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
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
                count: key.mesh.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("bevy_smud_pipeline".into()),
            push_constant_ranges: Vec::new(),
            zero_initialize_workgroup_memory: false,
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
#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

#import bevy_smud::view_bindings::view
#import smud

#import {sdf_import_path} as sdf
#import {fill_import_path} as fill

struct FragmentInput {{
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) params: vec4<f32>,
}};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {{
    let sdf_input = smud::SdfInput(in.pos, in.params);
    let d = sdf::sdf(sdf_input);
    let fill_input = smud::FillInput(
        in.pos,
        in.params,
        d,
        in.color,
    );
    var color = fill::fill(fill_input);

#ifdef TONEMAP_IN_SHADER
    color = tonemapping::tone_mapping(color, view.color_grading);
#endif

    return color;
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
    main_entity: Entity,
    render_entity: Entity,
    color: Color,
    params: Vec4,
    bounds: Vec2,
    sdf_shader: Handle<Shader>,
    fill_shader: Handle<Shader>,
    transform: GlobalTransform,
    blend_mode: BlendMode,
}

#[derive(Resource, Default, Debug)]
struct ExtractedShapes {
    shapes: Vec<ExtractedShape>,
}

#[allow(clippy::type_complexity)]
fn extract_shapes(
    mut extracted_shapes: ResMut<ExtractedShapes>,
    shape_query: Extract<
        Query<(
            Entity,
            RenderEntity,
            &ViewVisibility,
            &SmudShape,
            &GlobalTransform,
        )>,
    >,
) {
    extracted_shapes.shapes.clear();

    for (main_entity, render_entity, view_visibility, shape, transform) in shape_query.iter() {
        if !view_visibility.get() {
            continue;
        }

        // TODO: bevy_sprite has some slice stuff here? what is it for?

        extracted_shapes.shapes.push(ExtractedShape {
            main_entity,
            render_entity,
            color: shape.color,
            params: shape.params,
            transform: *transform,
            sdf_shader: shape.sdf.clone(),
            fill_shader: shape.fill.clone(),
            bounds: shape.bounds.half_size,
            blend_mode: shape.blend_mode,
        });
    }
}

// fork of Mesh2DPipelineKey (in order to remove bevy_sprite dependency)
// todo: merge with SmudPipelineKey?
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    // NOTE: Apparently quadro drivers support up to 64x MSAA.
    // MSAA uses the highest 3 bits for the MSAA log2(sample count) to support up to 128x MSAA.
    // FIXME: make normals optional?
    pub(crate) struct PipelineKey: u32 {
        const NONE                              = 0;
        const HDR                               = 1 << 0;
        const TONEMAP_IN_SHADER                 = 1 << 1;
        const DEBAND_DITHER                     = 1 << 2;
        const BLEND_ADDITIVE                    = 1 << 3;
        const MAY_DISCARD                       = 1 << 4;
        const MSAA_RESERVED_BITS                = Self::MSAA_MASK_BITS << Self::MSAA_SHIFT_BITS;
        const PRIMITIVE_TOPOLOGY_RESERVED_BITS  = Self::PRIMITIVE_TOPOLOGY_MASK_BITS << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
        const TONEMAP_METHOD_RESERVED_BITS      = Self::TONEMAP_METHOD_MASK_BITS << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_NONE               = 0 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_REINHARD           = 1 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_REINHARD_LUMINANCE = 2 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_ACES_FITTED        = 3 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_AGX                = 4 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM = 5 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_TONY_MC_MAPFACE    = 6 << Self::TONEMAP_METHOD_SHIFT_BITS;
        const TONEMAP_METHOD_BLENDER_FILMIC     = 7 << Self::TONEMAP_METHOD_SHIFT_BITS;
    }
}

impl PipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();
    const PRIMITIVE_TOPOLOGY_MASK_BITS: u32 = 0b111;
    const PRIMITIVE_TOPOLOGY_SHIFT_BITS: u32 = Self::MSAA_SHIFT_BITS - 3;
    const TONEMAP_METHOD_MASK_BITS: u32 = 0b111;
    const TONEMAP_METHOD_SHIFT_BITS: u32 =
        Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS - Self::TONEMAP_METHOD_MASK_BITS.count_ones();

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits =
            (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Self::from_bits_retain(msaa_bits)
    }

    pub fn from_hdr(hdr: bool) -> Self {
        if hdr { Self::HDR } else { Self::NONE }
    }

    pub fn msaa_samples(&self) -> u32 {
        1 << ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS)
    }

    pub fn from_primitive_topology(primitive_topology: PrimitiveTopology) -> Self {
        let primitive_topology_bits = ((primitive_topology as u32)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS)
            << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
        Self::from_bits_retain(primitive_topology_bits)
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

    pub fn from_blend_mode(blend_mode: crate::BlendMode) -> Self {
        match blend_mode {
            crate::BlendMode::Alpha => Self::NONE,
            crate::BlendMode::Additive => Self::BLEND_ADDITIVE,
        }
    }
}

#[allow(clippy::type_complexity)]
fn queue_shapes(
    mut view_entities: Local<FixedBitSet>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    smud_pipeline: Res<SmudPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline>>,
    pipeline_cache: ResMut<PipelineCache>,
    extracted_shapes: ResMut<ExtractedShapes>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut views: Query<(
        &RenderVisibleEntities,
        &ExtractedView,
        &Msaa,
        Option<&Tonemapping>,
        Option<&DebandDither>,
    )>,
    // ?
) {
    let draw_smud_shape_function = draw_functions.read().get_id::<DrawSmudShape>().unwrap();

    // Iterate over each view (a camera is a view)
    for (visible_entities, view, msaa, tonemapping, dither) in &mut views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let mesh_key = PipelineKey::from_msaa_samples(msaa.samples())
            | PipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        let mut view_key = PipelineKey::from_hdr(view.hdr) | mesh_key;

        if !view.hdr {
            if let Some(tonemapping) = tonemapping {
                view_key |= PipelineKey::TONEMAP_IN_SHADER;
                view_key |= match tonemapping {
                    Tonemapping::None => PipelineKey::TONEMAP_METHOD_NONE,
                    Tonemapping::Reinhard => PipelineKey::TONEMAP_METHOD_REINHARD,
                    Tonemapping::ReinhardLuminance => {
                        PipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE
                    }
                    Tonemapping::AcesFitted => PipelineKey::TONEMAP_METHOD_ACES_FITTED,
                    Tonemapping::AgX => PipelineKey::TONEMAP_METHOD_AGX,
                    Tonemapping::SomewhatBoringDisplayTransform => {
                        PipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM
                    }
                    Tonemapping::TonyMcMapface => PipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE,
                    Tonemapping::BlenderFilmic => PipelineKey::TONEMAP_METHOD_BLENDER_FILMIC,
                };
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= PipelineKey::DEBAND_DITHER;
            }
        }

        view_entities.clear();
        view_entities.extend(
            visible_entities
                .iter::<SmudShape>()
                .map(|(_, e)| e.index() as usize),
        );

        transparent_phase
            .items
            .reserve(extracted_shapes.shapes.len());

        for (index, extracted_shape) in extracted_shapes.shapes.iter().enumerate() {
            let shader = (
                extracted_shape.sdf_shader.id(),
                extracted_shape.fill_shader.id(),
            );

            let mut pipeline = CachedRenderPipelineId::INVALID;

            if let Some(_shader) = smud_pipeline.shaders.0.get(&shader) {
                // todo pass the shader into specialize
                let shape_key = view_key | PipelineKey::from_blend_mode(extracted_shape.blend_mode);
                let specialize_key = SmudPipelineKey {
                    mesh: shape_key,
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
                entity: (
                    extracted_shape.render_entity,
                    extracted_shape.main_entity.into(),
                ),
                sort_key,
                // batch_range and dynamic_offset will be calculated in prepare_shapes
                batch_range: 0..0,
                extra_index: PhaseItemExtraIndex::None,
                extracted_index: index,
                indexed: true,
            });
        }
    }
}

fn prepare_shape_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<(Entity, &Tonemapping), With<ExtractedView>>,
    tonemapping_luts: Res<TonemappingLuts>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let (Some(view_binding), Some(globals)) = (
        view_uniforms.uniforms.binding(),
        globals_buffer.buffer.binding(),
    ) else {
        return;
    };

    for (entity, tonemapping) in &views {
        let lut_bindings =
            get_lut_bindings(&images, &tonemapping_luts, tonemapping, &fallback_image);
        let view_bind_group = render_device.create_bind_group(
            "mesh2d_view_bind_group",
            &smud_pipeline.view_layout,
            &BindGroupEntries::with_indices((
                (0, view_binding.clone()),
                (1, globals.clone()),
                (2, lut_bindings.0),
                (3, lut_bindings.1),
            )),
        );

        commands.entity(entity).insert(ShapeViewBindGroup {
            value: view_bind_group,
        });
    }
}

fn prepare_shapes(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut shape_meta: ResMut<ShapeMeta>,
    extracted_shapes: Res<ExtractedShapes>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut batches: ResMut<ShapeBatches>,
) {
    batches.clear();

    // Clear the vertex buffer
    shape_meta.vertices.clear();

    // Vertex buffer index
    let mut index = 0;

    for (retained_view, transparent_phase) in phases.iter_mut() {
        let mut current_batch = None;
        let mut batch_item_index = 0;
        // let mut batch_image_size = Vec2::ZERO;
        // let mut batch_image_handle = AssetId::invalid();
        let mut batch_shader_handles = (AssetId::invalid(), AssetId::invalid());

        // Iterate through the phase items and detect when successive shapes that can be batched.
        // Spawn an entity with a `ShapeBatch` component for each possible batch.
        // Compatible items share the same entity.
        for item_index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[item_index];

            let Some(extracted_shape) = extracted_shapes
                .shapes
                .get(item.extracted_index)
                .filter(|extracted_shape| extracted_shape.render_entity == item.entity())
            else {
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

            let lrgba: LinearRgba = extracted_shape.color.into();
            let color = lrgba.to_f32_array();
            let params = extracted_shape.params.to_array();

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
                params,
                rotation,
                scale,
                bounds: extracted_shape.bounds.to_array(),
            };
            shape_meta.vertices.push(vertex);

            if batch_shader_changed {
                batch_item_index = item_index;

                current_batch = Some(batches.entry((*retained_view, item.main_entity())).insert(
                    ShapeBatch {
                        shader: shader_handles,
                        range: index..index,
                    },
                ));
            }

            transparent_phase.items[batch_item_index]
                .batch_range_mut()
                .end += 1;

            current_batch.as_mut().unwrap().get_mut().range.end += 1;
            index += 1;
        }
    }

    shape_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct ShapeVertex {
    pub color: [f32; 4],
    pub bounds: [f32; 2],
    pub params: [f32; 4], // for now all shapes have 4 f32 parameters
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
}

#[derive(Resource)]
pub(crate) struct ShapeMeta {
    vertices: RawBufferVec<ShapeVertex>,
}

impl Default for ShapeMeta {
    fn default() -> Self {
        Self {
            vertices: RawBufferVec::new(BufferUsages::VERTEX),
        }
    }
}

#[derive(Component)]
struct ShapeViewBindGroup {
    value: BindGroup,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ShapeBatches(HashMap<(RetainedViewEntity, MainEntity), ShapeBatch>);

#[derive(Component, Eq, PartialEq, Clone)]
struct ShapeBatch {
    shader: (AssetId<Shader>, AssetId<Shader>),
    range: Range<u32>,
}
