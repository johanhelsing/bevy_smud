use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{GpuBufferInfo, GpuMesh},
        render_component::UniformComponentPlugin,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BlendState, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites, Face,
            FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, SpecializedPipeline,
            SpecializedPipelines, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};

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

        // TODO: look at what it does!
        app.add_plugin(UniformComponentPlugin::<Mesh2dUniform>::default());

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            // TODO: what does it do internally?
            .add_render_command::<Transparent2d, DrawSmudShape>()
            .init_resource::<SmudPipeline>()
            .init_resource::<SpecializedPipelines<SmudPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_shapes)
            .add_system_to_stage(RenderStage::Queue, queue_shapes);
    }
}

type DrawSmudShape = (
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // DrawMesh2d,
    DrawQuad,
);

struct DrawQuad;
impl EntityRenderCommand for DrawQuad {
    type Param = SRes<SmudPipeline>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        pipeline: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_mesh = &pipeline.into_inner().quad;
        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..1);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..1);
            }
        }
        RenderCommandResult::Success
    }
}

#[derive(Component, Default)]
pub struct SmudShape;

struct SmudPipeline {
    mesh2d_pipeline: Mesh2dPipeline,
    quad: GpuMesh,
    // quad_handle: Handle<Mesh>,
}

impl FromWorld for SmudPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        let w = 100.;
        let v_pos = vec![[-w, -w], [w, -w], [-w, w], [w, w]];
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
        // And a RGB color attribute
        let v_color = vec![[0.5, 0.3, 0.1, 1.0]; 4];
        mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);
        // let indices = vec![0, 1, 2, 3];
        // quad.set_indices(Some(Indices::U32(indices)));

        // let quad = world
        //     .get_resource_mut::<Assets<Mesh>>()
        //     .unwrap()
        //     .add(quad.clone());
        let quad = {
            // let render_queue = world.get_resource_mut::<RenderQueue>().unwrap();
            let render_device = world.get_resource_mut::<RenderDevice>().unwrap();
            let vertex_buffer_data = mesh.get_vertex_buffer_data();
            let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::VERTEX,
                label: Some("Mesh Vertex Buffer"),
                contents: &vertex_buffer_data,
            });
            GpuMesh {
                vertex_buffer,
                buffer_info: GpuBufferInfo::NonIndexed { vertex_count: 4 },
                has_tangents: false,
                primitive_topology: mesh.primitive_topology(),
            }
        };

        Self {
            mesh2d_pipeline: FromWorld::from_world(world),
            // quad_handle: Default::default(), // this is initialized later when we can actually use Assets!
            quad,
        }
    }
}

impl SpecializedPipeline for SmudPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes(alphabetically))
            VertexAttribute {
                format: VertexFormat::Float32x2,
                // this offset is the size of the color attribute, which is stored first
                offset: 16,
                // position is available at location 0 in the shader
                shader_location: 0,
            },
            //Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
        ];
        // This is the sum of the size of position and color attributes (8 + 16 = 24)
        let vertex_array_stride = 24;

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
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
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

// TODO: what are they even used for?
// NOTE: These must match the bit flags in bevy_sprite/src/mesh2d/mesh2d.wgsl!
bitflags::bitflags! {
    #[repr(transparent)]
    struct MeshFlags: u32 {
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

fn extract_shapes(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility, &GlobalTransform), With<SmudShape>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility, transform) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        // TODO: copy over other data as well?
        let transform = transform.compute_matrix();
        values.push((
            entity,
            (
                SmudShape,
                Mesh2dUniform {
                    // TODO: what are the flags for?
                    flags: MeshFlags::empty().bits,
                    transform,
                    inverse_transpose_model: transform.inverse().transpose(),
                },
            ),
        ));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

fn queue_shapes(
    mut views: Query<(&mut RenderPhase<Transparent2d>, &VisibleEntities)>,
    mut pipelines: ResMut<SpecializedPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    smud_pipeline: Res<SmudPipeline>,
    shape_query: Query<&Mesh2dUniform, With<SmudShape>>,
    msaa: Res<Msaa>,
) {
    if shape_query.is_empty() {
        return;
    }
    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, visible_entities) in views.iter_mut() {
        let draw_smud_shape = transparent_draw_functions
            .read()
            .get_id::<DrawSmudShape>()
            .unwrap();

        // TODO: point of this key is?
        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
            | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        for visible_entity in &visible_entities.entities {
            if let Ok(mesh2d_uniform) = shape_query.get(*visible_entity) {
                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &smud_pipeline, mesh_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;

                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_smud_shape,
                    pipeline: pipeline_id,
                    sort_key: FloatOrd(mesh_z),
                    batch_range: None, // TODO: use this?
                });
            }
        }
    }
}
