use bevy::{
    asset::LoadState,
    core::FloatOrd,
    core_pipeline::Transparent2d,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            std140::{AsStd140, Std140},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferSize, BufferUsages, ColorTargetState,
            ColorWrites, Face, FragmentState, FrontFace, MultisampleState, PolygonMode,
            PrimitiveState, RenderPipelineCache, RenderPipelineDescriptor, ShaderStages,
            SpecializedPipeline, SpecializedPipelines, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Material2d, Material2dPipeline, Material2dPlugin, Mesh2dHandle, Mesh2dPipeline,
        Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};

pub struct SoSmoothPlugin;

const SMUD_MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5645555317811706725);

// Needed to keep the shaders alive
struct ShaderHandles {
    #[allow(dead_code)]
    smud: Handle<Shader>,
    prelude: Handle<Shader>,
    prelude_loaded: bool,
    colorize: Handle<Shader>,
    colorize_loaded: bool,
    shapes: Handle<Shader>,
    shapes_loaded: bool,
}

impl Plugin for SoSmoothPlugin {
    fn build(&self, app: &mut App) {
        // TODO: remove
        app.add_plugin(Material2dPlugin::<CustomMaterial>::default());

        let shaders = {
            let asset_server = app.world.get_resource::<AssetServer>().unwrap();
            ShaderHandles {
                smud: asset_server.load("smud.wgsl"),
                prelude: asset_server.load("prelude.wgsl"),
                prelude_loaded: false,
                colorize: asset_server.load("colorize.wgsl"),
                colorize_loaded: false,
                shapes: asset_server.load("shapes.wgsl"),
                shapes_loaded: false,
            }
        };

        app.world
            .get_resource_mut::<Assets<Shader>>()
            .unwrap()
            .add_alias(&shaders.smud, SMUD_MESH2D_SHADER_HANDLE);

        app.insert_resource(shaders);

        app.add_system_to_stage(CoreStage::PostUpdate, set_shader_import_paths);

        // non-hot-reload path
        // app.world
        //     .get_resource_mut::<Assets<Shader>>()
        //     .unwrap()
        //     .set_untracked(
        //         SMUD_MESH2D_SHADER_HANDLE,
        //         Shader::from_wgsl(include_str!("../assets/smud.wgsl")),
        //     );

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

fn set_shader_import_paths(
    asset_server: Res<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
    mut shader_handles: ResMut<ShaderHandles>,
) {
    if !shader_handles.prelude_loaded
        && asset_server.get_load_state(shader_handles.prelude.clone()) == LoadState::Loaded
    {
        shaders
            .get_mut(shader_handles.prelude.clone())
            .unwrap()
            .set_import_path("bevy_smud::prelude");
        info!("Prelude ready");
        shader_handles.prelude_loaded = true;
    }

    if !shader_handles.colorize_loaded
        && asset_server.get_load_state(shader_handles.colorize.clone()) == LoadState::Loaded
    {
        shaders
            .get_mut(shader_handles.colorize.clone())
            .unwrap()
            .set_import_path("bevy_smud::colorize");
        shader_handles.colorize_loaded = true;
    }

    if !shader_handles.shapes_loaded
        && asset_server.get_load_state(shader_handles.shapes.clone()) == LoadState::Loaded
    {
        shaders
            .get_mut(shader_handles.shapes.clone())
            .unwrap()
            .set_import_path("bevy_smud::shapes");
        shader_handles.shapes_loaded = true;
    }
}

type DrawSmudShape = (
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    DrawMesh2d,
);

#[derive(Debug, Clone, TypeUuid, AsStd140)]
#[uuid = "200dfa11-9b87-40e1-8774-ffcd3fcb17df"]
pub struct CustomMaterial {
    pub color: Vec4,
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl Material2d for CustomMaterial {
    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &bevy::render::renderer::RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(CustomMaterial::std140_size_static() as u64),
                },
                count: None, // TODO: what?
            }],
            label: None, // TODO: WHAT?
        })
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("custom_material.wgsl"))
    }
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (SRes<RenderDevice>, SRes<Material2dPipeline<Self>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let custom_material_std140 = extracted_asset.as_std140();
        let custom_material_bytes = custom_material_std140.as_bytes();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: custom_material_bytes,
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material2d_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

#[derive(Component, Default)]
pub struct SmudShape;

struct SmudPipeline {
    mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for SmudPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: FromWorld::from_world(world),
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
                shader: SMUD_MESH2D_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: SMUD_MESH2D_SHADER_HANDLE.typed::<Shader>(),
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

fn extract_shapes(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<SmudShape>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        // TODO: copy over other data as well?
        values.push((entity, (SmudShape,)));
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
    shape_query: Query<(&Mesh2dHandle, &Mesh2dUniform), With<SmudShape>>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
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

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = shape_query.get(*visible_entity) {
                // Get our specialized pipeline
                let mesh = render_meshes
                    .get(&mesh2d_handle.0)
                    .expect("Shape without mesh");
                let mesh_key =
                    mesh_key | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

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
