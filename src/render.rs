//! Renders `SmudShape`s as `Mesh2d` quads with a `Material2d`.
//!
//! Each shape gets a shared quad mesh and the material of its sdf, fill and
//! blend mode; the material's generated shader draws the pair. Its color,
//! params and bounds live in a `GpuComponentArrayBuffer`, indexed by the mesh
//! tag, and are uploaded only when the shape changes. Bevy retains the rest
//! between frames, so a shape that does not change costs next to nothing.

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibilitySystems},
    },
    ecs::{query::QueryItem, system::lifetimeless::Read},
    mesh::MeshVertexBufferLayoutRef,
    platform::collections::HashMap,
    prelude::*,
    render::{
        gpu_component_array_buffer::{
            GpuComponentArray, GpuComponentArrayBuffer, GpuComponentArrayBufferPlugin,
        },
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin},
};
use bytemuck::{Pod, Zeroable};

use crate::{
    BlendMode, ShapeRenderSystems, SmudShape,
    shader_loading::{import_path_of, wesl_module},
    util::generate_shader_id,
};

const SHAPE_SHADER_HANDLE: Handle<Shader> = uuid_handle!("8c3d6f35-2f4f-4b8e-9f43-6f0b1fb0b6f1");

pub(crate) struct ShapeRenderPlugin;

impl Plugin for ShapeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<SmudMaterial>::default(),
            GpuComponentArrayBufferPlugin::<SmudShape>::default(),
        ));
        load_internal_asset!(
            app,
            SHAPE_SHADER_HANDLE,
            "../assets/shape.wesl",
            wesl_module,
            "bevy_smud::shape"
        );
        app.init_resource::<ShapeMaterials>()
            .add_systems(Startup, add_quad)
            .add_systems(
                PostUpdate,
                prepare_shapes
                    .run_if(mesh2d_bindings_loaded)
                    .in_set(ShapeRenderSystems::PrepareShapes)
                    .before(VisibilitySystems::CalculateBounds),
            );
    }
}

/// The material of all shapes that share an sdf, a fill and a blend mode.
///
/// Added by [`crate::SmudPlugin`]; you don't make these yourself.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(SmudMaterialKey)]
pub struct SmudMaterial {
    /// Every shape's color, params and bounds, indexed by its mesh tag.
    #[storage(0, read_only)]
    instances: Handle<ShaderBuffer>,
    shader: Handle<Shader>,
    blend_mode: BlendMode,
}

/// Specializes [`SmudMaterial`]'s pipeline on its generated shader and blend mode.
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SmudMaterialKey {
    shader: Handle<Shader>,
    blend_mode: BlendMode,
}

impl From<&SmudMaterial> for SmudMaterialKey {
    fn from(material: &SmudMaterial) -> Self {
        Self {
            shader: material.shader.clone(),
            blend_mode: material.blend_mode,
        }
    }
}

const ADDITIVE_BLENDING: BlendState = BlendState {
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
};

impl Material2d for SmudMaterial {
    fn vertex_shader() -> ShaderRef {
        SHAPE_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![
            layout
                .0
                .get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?,
        ];
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = key.bind_group_data.shader;
            if key.bind_group_data.blend_mode == BlendMode::Additive {
                for target in fragment.targets.iter_mut().flatten() {
                    target.blend = Some(ADDITIVE_BLENDING);
                }
            }
        }
        Ok(())
    }
}

/// A shape's data as the shaders read it.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
#[doc(hidden)]
pub struct GpuSmudInstance {
    color: [f32; 4],
    params: [f32; 4],
    bounds: [f32; 4],
}

impl GpuComponentArrayBuffer for SmudShape {
    type QueryData = Read<SmudShape>;
    type QueryFilter = Changed<SmudShape>;
    type Out = GpuSmudInstance;

    fn extract_component(shape: QueryItem<'_, '_, Self::QueryData>) -> Option<Self::Out> {
        let half_size = shape.bounds.half_size;
        Some(GpuSmudInstance {
            color: shape.color.to_linear().to_f32_array(),
            params: shape.params.to_array(),
            bounds: [half_size.x, half_size.y, shape.extra_bounds, 0.],
        })
    }
}

#[derive(Resource, Default)]
struct ShapeMaterials {
    quad: Handle<Mesh>,
    shaders: HashMap<(AssetId<Shader>, AssetId<Shader>), Handle<Shader>>,
    materials: HashMap<(AssetId<Shader>, AssetId<Shader>, BlendMode), Handle<SmudMaterial>>,
}

/// Whether Bevy has loaded the `mesh2d::bindings` shader module that the shape
/// shaders import.
///
/// Bevy loads it at render startup, once it knows the device's limits. A shape
/// pipeline compiled before that logs an unresolved import (bevy#25936), so
/// shapes get their materials only after it.
fn mesh2d_bindings_loaded(asset_server: Res<AssetServer>, mut loaded: Local<bool>) -> bool {
    *loaded = *loaded
        || asset_server
            .get_handle::<Shader>("embedded://bevy_sprite_render/mesh2d/bindings.wesl")
            .is_some_and(|handle| asset_server.is_loaded(&handle));
    *loaded
}

fn add_quad(mut shape_materials: ResMut<ShapeMaterials>, mut meshes: ResMut<Assets<Mesh>>) {
    shape_materials.quad = meshes.add(Rectangle::new(2., 2.));
}

/// Gives each new or changed shape the quad, the material of its sdf, fill and
/// blend mode, and its bounds. Generates a pair's shader the first time it is
/// seen; a shape whose sdf or fill is still loading waits for it.
#[allow(clippy::type_complexity)]
fn prepare_shapes(
    mut commands: Commands,
    mut shape_materials: ResMut<ShapeMaterials>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<SmudMaterial>>,
    instances: Res<GpuComponentArray<SmudShape>>,
    shapes: Query<
        (
            Entity,
            &SmudShape,
            Option<&MeshMaterial2d<SmudMaterial>>,
            Option<&Aabb>,
        ),
        Or<(Changed<SmudShape>, Without<MeshMaterial2d<SmudMaterial>>)>,
    >,
) {
    let ShapeMaterials {
        quad,
        shaders: generated,
        materials: shape_materials,
    } = &mut *shape_materials;
    for (entity, shape, current_material, current_aabb) in &shapes {
        let key = (shape.sdf.id(), shape.fill.id(), shape.blend_mode);
        let material = match shape_materials.get(&key) {
            Some(material) => material.clone(),
            None => {
                let shader = match generated.get(&(key.0, key.1)) {
                    Some(shader) => shader.clone(),
                    None => {
                        let Some(shader) = generate_shader(&mut shaders, &shape.sdf, &shape.fill)
                        else {
                            continue;
                        };
                        generated.insert((key.0, key.1), shader.clone());
                        shader
                    }
                };
                let material = materials.add(SmudMaterial {
                    instances: instances.buffer.clone(),
                    shader,
                    blend_mode: shape.blend_mode,
                });
                shape_materials.insert(key, material.clone());
                material
            }
        };

        // The quad is scaled in the vertex shader, so Bevy's bounds for it are
        // wrong for culling.
        let half_extents = shape.bounds.half_size + Vec2::splat(shape.extra_bounds);
        let aabb = Aabb::from_min_max((-half_extents).extend(0.), half_extents.extend(0.));

        if current_material.is_none_or(|current| current.0 != material) {
            commands.entity(entity).insert((
                Mesh2d(quad.clone()),
                MeshMaterial2d(material),
                NoAutoAabb,
                aabb,
            ));
        } else if current_aabb != Some(&aabb) {
            commands.entity(entity).insert(aabb);
        }
    }
}

fn generate_shader(
    shaders: &mut Assets<Shader>,
    sdf: &Handle<Shader>,
    fill: &Handle<Shader>,
) -> Option<Handle<Shader>> {
    let sdf_import_path = import_path_of(shaders, sdf, "sdf")?;
    let fill_import_path = import_path_of(shaders, fill, "fill")?;
    let source = format!(
        r#"
import bevy_smud::shape::{{VertexOutput, instance}};
import bevy_sprite_render::mesh2d::view_bindings::view;
@if(TONEMAP_IN_SHADER)
import bevy_core_pipeline::tonemapping;
@if(SRGB_OUTPUT)
import bevy_render::color_operations::linear_to_srgb;
@if(OKLAB_OUTPUT)
import bevy_render::color_operations::linear_rgb_to_oklab;

import {sdf_import_path} as sdf;
import {fill_import_path} as fill;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {{
    let data = instance(in.instance_index);
    let d = sdf::sdf(smud::SdfInput(in.pos, data.params, data.bounds.xy));
    var color = fill::fill(smud::FillInput(in.pos, data.params, d, data.color));

@if(TONEMAP_IN_SHADER)
    color = tonemapping::tone_mapping(color, view.color_grading);

@if(SRGB_OUTPUT)
    color = vec4(linear_to_srgb(color.rgb), color.a);

@if(OKLAB_OUTPUT)
    color = vec4(linear_rgb_to_oklab(color.rgb), color.a);

    return color;
}}
"#
    );
    Some(shaders.add(Shader::from_wesl(
        source,
        // An asset path, so `package::` in the imports names the asset root.
        format!("bevy_smud/generated/shape{}.wesl", generate_shader_id()),
    )))
}
