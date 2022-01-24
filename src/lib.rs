use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    math::const_vec2,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_phase::{
            AddRenderCommand, BatchedPhaseItem, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            std140::AsStd140, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, BufferSize, BufferUsages, BufferVec, ColorTargetState, ColorWrites,
            Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, ShaderStages,
            SpecializedPipeline, SpecializedPipelines, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ViewUniform, ViewUniformOffset, ViewUniforms, VisibleEntities},
        RenderApp, RenderStage, RenderWorld,
    },
    sprite::Mesh2dPipelineKey,
};
use bytemuck::{Pod, Zeroable};
use copyless::VecHelper;

mod bundle;
mod components;

pub use bundle::ShapeBundle;
pub use components::*;

#[derive(Default)]
pub struct SoSmoothPlugin;

const PRELUDE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11291576006157771079);
const PRELUDE_SHADER_IMPORT: &str = "bevy_smud::prelude";

const SHAPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10055894596049459186);
const SHAPES_SHADER_IMPORT: &str = "bevy_smud::shapes";

const COLORIZE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10050447940405429418);
const COLORIZE_SHADER_IMPORT: &str = "bevy_smud::colorize";

const SMUD_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5645555317811706725);
const SMUD_SHADER_IMPORT: &str = "bevy_smud::smud";

#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShader {
    strong_handle: Handle<Shader>,
    untyped_handle: Option<HandleUntyped>,
    loaded: bool,
    import_path: String,
}

// Needed to keep the shaders alive
#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShaders<T> {
    shaders: Vec<HotShader>,
    marker: std::marker::PhantomData<T>,
}

#[cfg(feature = "smud_shader_hot_reloading")]
impl<T> Default for HotShaders<T> {
    fn default() -> Self {
        Self {
            shaders: Default::default(),
            marker: Default::default(),
        }
    }
}

#[cfg(feature = "smud_shader_hot_reloading")]
fn setup_shader_imports<T: 'static + Send + Sync>(
    mut hot_shaders: ResMut<HotShaders<T>>,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    for hot_shader in hot_shaders.shaders.iter_mut() {
        if !hot_shader.loaded
            && asset_server.get_load_state(hot_shader.strong_handle.clone())
                == bevy::asset::LoadState::Loaded
        {
            shaders
                .get_mut(hot_shader.strong_handle.clone())
                .unwrap()
                .set_import_path(&hot_shader.import_path);

            hot_shader.loaded = true;
        }
    }
}

impl Plugin for SoSmoothPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "smud_shader_hot_reloading")]
        {
            let mut hot_shaders = {
                let asset_server = app.world.get_resource::<AssetServer>().unwrap();
                HotShaders::<Self> {
                    shaders: [
                        ("prelude.wgsl", PRELUDE_SHADER_IMPORT, PRELUDE_SHADER_HANDLE),
                        ("shapes.wgsl", SHAPES_SHADER_IMPORT, SHAPES_SHADER_HANDLE),
                        (
                            "colorize.wgsl",
                            COLORIZE_SHADER_IMPORT,
                            COLORIZE_SHADER_HANDLE,
                        ),
                        ("smud.wgsl", SMUD_SHADER_IMPORT, SMUD_SHADER_HANDLE),
                    ]
                    .into_iter()
                    .map(|(path, import_path, untyped_handle)| HotShader {
                        strong_handle: asset_server.load(path),
                        untyped_handle: Some(untyped_handle),
                        import_path: import_path.into(),
                        loaded: false,
                    })
                    .collect(),
                    ..Default::default()
                }
            };
            let mut shader_assets = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            for hot_shader in hot_shaders.shaders.iter_mut() {
                let untyped_handle = hot_shader.untyped_handle.take().unwrap();
                shader_assets.add_alias(hot_shader.strong_handle.clone(), untyped_handle);
            }

            app.insert_resource(hot_shaders);
            app.add_system(setup_shader_imports::<SoSmoothPlugin>);
        }

        #[cfg(not(feature = "smud_shader_hot_reloading"))]
        {
            let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            let prelude = Shader::from_wgsl(include_str!("../assets/prelude.wgsl"))
                .with_import_path(PRELUDE_SHADER_IMPORT);
            shaders.set_untracked(PRELUDE_SHADER_HANDLE, prelude);

            let shapes = Shader::from_wgsl(include_str!("../assets/shapes.wgsl"))
                .with_import_path(SHAPES_SHADER_IMPORT);
            shaders.set_untracked(SHAPES_SHADER_HANDLE, shapes);

            let colorize = Shader::from_wgsl(include_str!("../assets/colorize.wgsl"))
                .with_import_path(COLORIZE_SHADER_IMPORT);
            shaders.set_untracked(COLORIZE_SHADER_HANDLE, colorize);

            let smud = Shader::from_wgsl(include_str!("../assets/smud.wgsl"))
                .with_import_path(SMUD_SHADER_IMPORT);
            shaders.set_untracked(SMUD_SHADER_HANDLE, smud);
        }

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape>()
                .init_resource::<ExtractedShapes>()
                .init_resource::<ShapeMeta>()
                .init_resource::<SmudPipeline>()
                .init_resource::<SpecializedPipelines<SmudPipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_shapes)
                .add_system_to_stage(RenderStage::Queue, queue_shapes);
        }
    }
}

type DrawSmudShape = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    // SetSpriteTextureBindGroup<1>,
    DrawShapeBatch,
);
struct SetShapeViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetShapeViewBindGroup<I> {
    type Param = (SRes<ShapeMeta>, SQuery<Read<ViewUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (sprite_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            sprite_meta.into_inner().view_bind_group.as_ref().unwrap(),
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
        // let sprite_batch = query_batch.get(item.entity()).unwrap();
        let shape_meta = shape_meta.into_inner();
        pass.set_vertex_buffer(0, shape_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
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
            label: Some("sprite_view_layout"),
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
            // quad_handle: Default::default(), // this is initialized later when we can actually use Assets!
            // quad,
        }
    }
}

impl SpecializedPipeline for SmudPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // (GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 2,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 4 * 4,
                shader_location: 0,
            },
            // UV
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 4 * 4 + 4 * 3,
                shader_location: 1,
            },
        ];
        // This is the sum of the size of position and color attributes (8 + 16 = 24)
        // let vertex_array_stride = 24;
        let vertex_array_stride = 4 * 4 + 4 * 3 + 4 * 2;

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
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
                topology: key.primitive_topology(),
                strip_index_format: None, // TODO: what does this do?
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("smud_pipeline".into()),
        }
    }
}

#[derive(Component, Clone, Copy)]
struct ExtractedShape {
    transform: GlobalTransform,
    color: Color,
}

#[derive(Default)]
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

        extracted_shapes.0.alloc().init(ExtractedShape {
            color: shape.color,
            transform: *transform,
            // rect: None,
            // // Pass the custom size
            // custom_size: sprite.custom_size,
            // image_handle_id: handle.id,
        });
    }
}

const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    const_vec2!([-0.5, -0.5]),
    const_vec2!([0.5, -0.5]),
    const_vec2!([0.5, 0.5]),
    const_vec2!([-0.5, 0.5]),
];

const QUAD_UVS: [Vec2; 4] = [
    const_vec2!([-1., 1.]),
    const_vec2!([1., 1.]),
    const_vec2!([1., -1.]),
    const_vec2!([-1., -1.]),
];

fn queue_shapes(
    mut commands: Commands,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &VisibleEntities)>,
    mut pipelines: ResMut<SpecializedPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
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

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
            | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);

        let pipeline_id = pipelines.specialize(&mut pipeline_cache, &smud_pipeline, mesh_key);

        // Everything is one batch for now
        let current_batch = ShapeBatch {};
        let current_batch_entity = commands.spawn_bundle((current_batch,)).id();

        // Add a phase item for each shape, and detect when successive items can be batched.
        // Spawn an entity with a `ShapeBatch` component for each possible batch.
        // Compatible items share the same entity.
        // Batches are merged later (in `batch_phase_system()`), so that they can be interrupted
        // by any other phase item (and they can interrupt other items from batching).
        for extracted_shape in extracted_shapes.iter() {
            // let mesh_z = mesh2d_uniform.transform.w_axis.z;

            let quad_size = 30.; // todo

            // Apply size and global transform
            let positions = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                extracted_shape
                    .transform
                    .mul_vec3((quad_pos * quad_size).extend(0.))
                    .into()
            });

            // let color = extracted_shape.color.as_linear_rgba_f32();
            // // encode color as a single u32 to save space
            // let color = (color[0] * 255.0) as u32
            //     | ((color[1] * 255.0) as u32) << 8
            //     | ((color[2] * 255.0) as u32) << 16
            //     | ((color[3] * 255.0) as u32) << 24;

            let color = extracted_shape.color.as_linear_rgba_f32();

            for i in QUAD_INDICES.iter() {
                let vertex = ShapeVertex {
                    position: positions[*i],
                    uv: QUAD_UVS[*i].into(), // todo: can be moved into shader?
                    color,
                };
                shape_meta.vertices.push(vertex);
            }

            let item_start = index;
            index += QUAD_INDICES.len() as u32;
            let item_end = index;

            transparent_phase.add(Transparent2d {
                entity: current_batch_entity,
                draw_function: draw_smud_shape,
                pipeline: pipeline_id,
                sort_key: FloatOrd(0.), // todo
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
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

pub struct ShapeMeta {
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

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct ShapeBatch {}
