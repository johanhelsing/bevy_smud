use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PolygonMode, PrimitiveState, RenderPipelineCache,
            RenderPipelineDescriptor, SpecializedPipeline, SpecializedPipelines, TextureFormat,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};

pub struct SoSmoothPlugin;

const SMUD_MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5645555317811706725);

// Needed to keep the shaders alive
#[cfg(feature = "smud_shader_hot_reloading")]
struct ShaderHandles {
    #[allow(dead_code)]
    smud: Handle<Shader>,
}

impl Plugin for SoSmoothPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "smud_shader_hot_reloading")]
        {
            let shaders = {
                let asset_server = app.world.get_resource::<AssetServer>().unwrap();
                ShaderHandles {
                    smud: asset_server.load("smud.wgsl"),
                }
            };

            app.world
                .get_resource_mut::<Assets<Shader>>()
                .unwrap()
                .add_alias(&shaders.smud, SMUD_MESH2D_SHADER_HANDLE);

            app.insert_resource(shaders);
        }

        #[cfg(not(feature = "smud_shader_hot_reloading"))]
        {
            app.world
                .get_resource_mut::<Assets<Shader>>()
                .unwrap()
                .set_untracked(
                    SMUD_MESH2D_SHADER_HANDLE,
                    Shader::from_wgsl(include_str!("../assets/smud.wgsl")),
                );
        }

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
    DrawMesh2d,
);

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
