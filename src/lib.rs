//! See the [readme](https://github.com/johanhelsing/bevy_smud) and
//! [examples](https://github.com/johanhelsing/bevy_smud/tree/main/examples) for
//! usage.

#![allow(clippy::too_many_arguments)]

use std::cmp::Ordering;

use bevy::{
    asset::HandleId,
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    math::Vec3Swizzles,
    prelude::*,
    reflect::Uuid,
    render::{
        render_phase::{
            AddRenderCommand, BatchedPhaseItem, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            std140::AsStd140, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, BufferSize, BufferUsages, BufferVec, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, ShaderImport, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ViewUniform, ViewUniformOffset, ViewUniforms, VisibleEntities},
        RenderApp, RenderStage, RenderWorld,
    },
    sprite::Mesh2dPipelineKey,
    utils::HashMap,
};
use bytemuck::{Pod, Zeroable};
use copyless::VecHelper;
use shader_loading::*;
use ui::UiShapePlugin;

pub use bundle::{ShapeBundle, UiShapeBundle};
pub use components::*;
pub use shader_loading::{DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE};

mod bundle;
mod components;
mod sdf_assets;
mod shader_loading;
mod ui;

#[cfg(feature = "bevy-inspector-egui")]
mod inspectable_plugin;

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_smud::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        sdf_assets::SdfAssets, Frame, ShapeBundle, SmudPlugin, SmudShape, UiShapeBundle,
        DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE,
    };
}

#[derive(Default)]
/// Main plugin for enabling rendering of Sdf shapes
pub struct SmudPlugin;

impl Plugin for SmudPlugin {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugin(ShaderLoadingPlugin);
        app.add_plugin(UiShapePlugin);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape>()
                .init_resource::<ExtractedShapes>()
                .init_resource::<ShapeMeta>()
                .init_resource::<SmudPipeline>()
                .init_resource::<SpecializedRenderPipelines<SmudPipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_shapes)
                .add_system_to_stage(RenderStage::Extract, extract_sdf_shaders)
                .add_system_to_stage(RenderStage::Queue, queue_shapes);
        }

        #[cfg(feature = "bevy-inspector-egui")]
        app.add_plugin(inspectable_plugin::InspectablePlugin);
    }
}

type DrawSmudShape = (SetItemPipeline, SetShapeViewBindGroup<0>, DrawShapeBatch);
struct SetShapeViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetShapeViewBindGroup<I> {
    type Param = (SRes<ShapeMeta>, SQuery<Read<ViewUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (shape_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            shape_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawShapeBatch;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawShapeBatch {
    type Param = (SRes<ShapeMeta>, SQuery<Read<ShapeBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &P,
        (shape_meta, _query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // let shape_batch = query_batch.get(item.entity()).unwrap();
        let shape_meta = shape_meta.into_inner();
        pass.set_vertex_buffer(0, shape_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(0..4, item.batch_range().as_ref().unwrap().clone());
        RenderCommandResult::Success
    }
}

// struct DrawQuad;
// impl EntityRenderCommand for DrawQuad {
//     type Param = SRes<SmudPipeline>;
//     #[inline]
//     fn render<'w>(
//         _view: Entity,
//         _item: Entity,
//         pipeline: SystemParamItem<'w, '_, Self::Param>,
//         pass: &mut TrackedRenderPass<'w>,
//     ) -> RenderCommandResult {
//         let gpu_mesh = &pipeline.into_inner().quad;
//         pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
//         match &gpu_mesh.buffer_info {
//             GpuBufferInfo::Indexed {
//                 buffer,
//                 index_format,
//                 count,
//             } => {
//                 pass.set_index_buffer(buffer.slice(..), 0, *index_format);
//                 pass.draw_indexed(0..*count, 0, 0..1);
//             }
//             GpuBufferInfo::NonIndexed { vertex_count } => {
//                 pass.draw(0..*vertex_count, 0..1);
//             }
//         }
//         RenderCommandResult::Success
//     }
// }

struct SmudPipeline {
    view_layout: BindGroupLayout,
    shaders: ShapeShaders,
}

impl FromWorld for SmudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
                },
                count: None,
            }],
            label: Some("shape_view_layout"),
        });
        // let quad = {
        //     let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        //     let w = 0.5;
        //     let v_pos = vec![[-w, -w], [w, -w], [-w, w], [w, w]];
        //     mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
        //     let v_color = vec![[0.5, 0.3, 0.1, 1.0]; 4];
        //     mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

        //     let render_device = world.get_resource_mut::<RenderDevice>().unwrap();
        //     let vertex_buffer_data = mesh.get_vertex_buffer_data();
        //     let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        //         usage: BufferUsages::VERTEX,
        //         label: Some("Mesh Vertex Buffer"),
        //         contents: &vertex_buffer_data,
        //     });
        //     GpuMesh {
        //         vertex_buffer,
        //         buffer_info: GpuBufferInfo::NonIndexed { vertex_count: 4 },
        //         has_tangents: false,
        //         primitive_topology: mesh.primitive_topology(),
        //     }
        // };

        Self {
            view_layout,
            shaders: default()
            // quad_handle: default(), // this is initialized later when we can actually use Assets!
            // quad,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SmudPipelineKey {
    mesh: Mesh2dPipelineKey,
    shader: (HandleId, HandleId),
}

impl SpecializedRenderPipeline for SmudPipeline {
    type Key = SmudPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader = self.shaders.0.get(&key.shader).unwrap();
        debug!("specializing for {shader:?}");

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
            // Frame
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4) * 4,
                shader_location: 5,
            },
            // perf: Maybe it's possible to pack this more efficiently?
            // Params
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (4 + 1) * 4,
                shader_location: 2,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: (4 + 1 + 4) * 4,
                shader_location: 0,
            },
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: (4 + 1 + 4 + 3) * 4,
                shader_location: 3,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4 + 1 + 4 + 3 + 2) * 4,
                shader_location: 4,
            },
        ];
        // This is the sum of the size of the attributes above
        let vertex_array_stride = (4 + 1 + 4 + 3 + 2 + 1) * 4;

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: VERTEX_SHADER_HANDLE.typed(),
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
                // shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.view_layout.clone(),
            ]),
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
        }
    }
}

#[derive(Default)]
struct ShapeShaders(HashMap<(HandleId, HandleId), Handle<Shader>>);

fn extract_sdf_shaders(
    mut render_world: ResMut<RenderWorld>,
    shapes: Query<&SmudShape>, //, Changed<SmudShape>>, // does changed help? need to make sure it is not racy then!
    mut shaders: ResMut<Assets<Shader>>,
) {
    let mut pipeline = render_world.get_resource_mut::<SmudPipeline>().unwrap();

    for shape in shapes.iter() {
        let shader_key = (shape.sdf.id, shape.fill.id);
        if pipeline.shaders.0.contains_key(&shader_key) {
            continue;
        }

        // todo use asset events instead?
        let sdf_import_path = match shaders.get_mut(&shape.sdf.clone()) {
            Some(shader) => match shader.import_path() {
                Some(ShaderImport::Custom(p)) => p.to_owned(),
                _ => {
                    let id = Uuid::new_v4();
                    let path = format!("bevy_smud::generated::{id}");
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
                Some(ShaderImport::Custom(p)) => p.to_owned(),
                _ => {
                    let id = Uuid::new_v4();
                    let path = format!("bevy_smud::generated::{id}");
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
        let generated_shader = Shader::from_wgsl(format!(
            r#"
#import {sdf_import_path}
#import {fill_import_path}
#import bevy_smud::fragment
"#
        ));

        // todo does this work, or is it too late?
        let generated_shader_handle = shaders.add(generated_shader);

        pipeline
            .shaders
            .0
            .insert(shader_key, generated_shader_handle);
    }
}

#[derive(Component, Clone, Debug)]
struct ExtractedShape {
    color: Color,
    params: Option<Vec4>,
    frame: f32,
    sdf_shader: Handle<Shader>,  // todo could be HandleId?
    fill_shader: Handle<Shader>, // todo could be HandleId?
    transform: GlobalTransform,
}

#[derive(Default, Debug)]
struct ExtractedShapes(Vec<ExtractedShape>);

fn extract_shapes(
    mut render_world: ResMut<RenderWorld>,
    query: Query<(&SmudShape, &ComputedVisibility, &GlobalTransform)>,
) {
    let mut extracted_shapes = render_world.get_resource_mut::<ExtractedShapes>().unwrap();
    extracted_shapes.0.clear();

    for (shape, computed_visibility, transform) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }

        let Frame::Quad(frame) = shape.frame;

        extracted_shapes.0.alloc().init(ExtractedShape {
            color: shape.color,
            params: shape.params,
            transform: *transform,
            sdf_shader: shape.sdf.clone_weak(),
            fill_shader: shape.fill.clone_weak(),
            frame
            // rect: None,
            // // Pass the custom size
            // custom_size: shape.custom_size,
        });
    }
}

fn queue_shapes(
    mut commands: Commands,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &VisibleEntities)>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut extracted_shapes: ResMut<ExtractedShapes>, // todo needs mut?
    mut shape_meta: ResMut<ShapeMeta>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    msaa: Res<Msaa>,
    view_uniforms: Res<ViewUniforms>,
    render_queue: Res<RenderQueue>,
) {
    // Clear the vertex buffer
    shape_meta.vertices.clear();

    let view_binding = match view_uniforms.uniforms.binding() {
        Some(binding) => binding,
        None => return,
    };

    shape_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
        label: Some("smud_shape_view_bind_group"),
        layout: &smud_pipeline.view_layout,
    }));

    // Vertex buffer index
    let mut index = 0;

    let draw_smud_shape = transparent_draw_functions
        .read()
        .get_id::<DrawSmudShape>()
        .unwrap();

    let shape_meta = &mut shape_meta;

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, _visible_entities) in views.iter_mut() {
        // todo: check visible entities?

        let extracted_shapes = &mut extracted_shapes.0;

        // Sort shapes by z for correct transparency and then by handle to improve batching
        extracted_shapes.sort_unstable_by(|a, b| {
            match a
                .transform
                .translation
                .z
                .partial_cmp(&b.transform.translation.z)
            {
                Some(Ordering::Equal) | None => {
                    (&a.sdf_shader, &a.fill_shader).cmp(&(&b.sdf_shader, &b.fill_shader))
                }
                Some(other) => other,
            }
        });

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
            | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        // Impossible starting values that will be replaced on the first iteration
        let mut current_batch = ShapeBatch {
            shader: (
                HandleId::Id(Uuid::nil(), u64::MAX),
                HandleId::Id(Uuid::nil(), u64::MAX),
            ),
        };
        let mut current_batch_entity = Entity::from_raw(u32::MAX);
        let mut current_batch_pipeline = CachedRenderPipelineId::INVALID;

        // Add a phase item for each shape, and detect when successive items can be batched.
        // Spawn an entity with a `ShapeBatch` component for each possible batch.
        // Compatible items share the same entity.
        // Batches are merged later (in `batch_phase_system()`), so that they can be interrupted
        // by any other phase item (and they can interrupt other items from batching).
        for extracted_shape in extracted_shapes.iter() {
            let new_batch = ShapeBatch {
                shader: (
                    extracted_shape.sdf_shader.id,
                    extracted_shape.fill_shader.id,
                ),
            };

            if new_batch != current_batch {
                current_batch_entity = commands.spawn_bundle((current_batch,)).id();

                current_batch = new_batch;

                if let Some(_shader) = smud_pipeline.shaders.0.get(&current_batch.shader) {
                    // todo pass the shader into specialize
                    let specialize_key = SmudPipelineKey {
                        mesh: mesh_key,
                        shader: current_batch.shader,
                    };
                    current_batch_pipeline =
                        pipelines.specialize(&mut pipeline_cache, &smud_pipeline, specialize_key);
                }
            }

            if current_batch_pipeline == CachedRenderPipelineId::INVALID {
                debug!("Shape not ready yet, skipping");
                continue; // skip shapes that are not ready yet
            }

            // let mesh_z = mesh2d_uniform.transform.w_axis.z;

            // let color = extracted_shape.color.as_linear_rgba_f32();
            // // encode color as a single u32 to save space
            // let color = (color[0] * 255.0) as u32
            //     | ((color[1] * 255.0) as u32) << 8
            //     | ((color[2] * 255.0) as u32) << 16
            //     | ((color[3] * 255.0) as u32) << 24;

            let color = extracted_shape.color.as_linear_rgba_f32();
            let params = extracted_shape.params.unwrap_or_default().to_array();

            let position = extracted_shape.transform.translation;
            let z = position.z;
            let position = position.into();

            let rotation = extracted_shape.transform.rotation * Vec3::X;
            let rotation = rotation.xy().into();

            let vertex = ShapeVertex {
                position,
                color,
                params,
                rotation,
                scale: extracted_shape.transform.scale.x,
                frame: extracted_shape.frame,
            };
            shape_meta.vertices.push(vertex);

            let item_start = index;
            index += 1;
            let item_end = index;

            transparent_phase.add(Transparent2d {
                entity: current_batch_entity,
                draw_function: draw_smud_shape,
                pipeline: current_batch_pipeline,
                sort_key: FloatOrd(z),
                batch_range: Some(item_start..item_end),
            });
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
    pub frame: f32,
    pub params: [f32; 4], // for now all shapes have 4 f32 parameters
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
    // pub uv: [f32; 2],
}

pub struct ShapeMeta {
    vertices: BufferVec<ShapeVertex>,
    ui_vertices: BufferVec<ShapeVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for ShapeMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            ui_vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct ShapeBatch {
    shader: (HandleId, HandleId),
}

// TODO: is RenderAsset asking too much?
// pub trait SdfShapeShader: 'static + Send + Sync {
//     /// Shader must include a handle to a shader with a wgsl function with signature fn distance(pos: vec2<f32>) -> f32
//     fn shader(asset_server: &AssetServer) -> Handle<Shader>;
// }

// /// Adds the necessary ECS resources and render logic to enable rendering entities using the given [`SdfShapeShader`]
// /// asset type
// pub struct SdfShapePlugin<S: SdfShapeShader>(PhantomData<S>);

// impl<S: SdfShapeShader> Default for SdfShapePlugin<S> {
//     fn default() -> Self {
//         Self(default())
//     }
// }

// impl<S: SdfShapeShader> Plugin for SdfShapePlugin<S> {
//     fn build(&self, app: &mut App) {
//         // TODO:
//         // app.add_asset::<S>()
//         //     .add_plugin(ExtractComponentPlugin::<Handle<S>>::default())
//         //     .add_plugin(RenderAssetPlugin::<S>::default());
//         // if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
//         //     render_app
//         //         .add_render_command::<Transparent3d, DrawMaterial<S>>()
//         //         .add_render_command::<Opaque3d, DrawMaterial<S>>()
//         //         .add_render_command::<AlphaMask3d, DrawMaterial<S>>()
//         //         .init_resource::<MaterialPipeline<S>>()
//         //         .init_resource::<SpecializedPipelines<MaterialPipeline<S>>>()
//         //         .add_system_to_stage(RenderStage::Queue, queue_material_meshes::<S>);
//         // }
//     }
// }
