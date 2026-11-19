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
            )
            // After `PostUpdate`, where the instance buffer drops the shape,
            // and before the frame is drawn.
            .add_systems(Last, drop_removed_shapes);
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

/// Takes what [`prepare_shapes`] gave an entity off it once its shape is
/// gone, so it stops drawing.
///
/// Removing a [`SmudShape`] frees its slot in the instance buffer, and the
/// last shape's data is moved into that slot. With its mesh and material
/// left on, the entity would go on drawing its sdf with another shape's
/// color, params and bounds. A shape put back before this runs keeps them.
/// A `Mesh2d` that is not the shapes' quad belongs to someone else (a
/// sprite) and stays.
///
/// The [`MeshTag`](bevy::mesh::MeshTag) stays too: the buffer owns it, and only the buffer knows
/// when the slot is freed, which may be a frame after the removal. Taken
/// off before that, a shape put back next frame is pushed again while the
/// buffer still holds it.
fn drop_removed_shapes(
    mut removed: RemovedComponents<SmudShape>,
    entities: Query<(Has<SmudShape>, Option<&Mesh2d>)>,
    shape_materials: Res<ShapeMaterials>,
    mut commands: Commands,
) {
    for entity in removed.read() {
        let Ok((has_shape, mesh)) = entities.get(entity) else {
            continue; // despawned
        };
        if has_shape {
            continue;
        }
        let mut entity = commands.entity(entity);
        entity.try_remove::<(MeshMaterial2d<SmudMaterial>, NoAutoAabb, Aabb)>();
        if mesh.is_some_and(|mesh| mesh.0 == shape_materials.quad) {
            entity.try_remove::<Mesh2d>();
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

#[cfg(test)]
mod tests {
    use bevy::{app::TaskPoolPlugin, mesh::MeshTag};

    use super::*;

    /// The instance buffer as the renderer keeps it, and what
    /// [`prepare_shapes`] gives a shape, without a renderer.
    fn app() -> (App, Handle<Mesh>) {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
            .init_asset::<ShaderBuffer>()
            .add_plugins(GpuComponentArrayBufferPlugin::<SmudShape>::default());
        let quad = Handle::<Mesh>::default();
        app.insert_resource(ShapeMaterials {
            quad: quad.clone(),
            ..default()
        })
        .add_systems(Last, drop_removed_shapes);
        (app, quad)
    }

    /// A shape told apart from the others by its first param.
    fn shape(id: f32) -> SmudShape {
        SmudShape {
            params: Vec4::new(id, 0., 0., 0.),
            ..default()
        }
    }

    fn drawn_shape(id: f32, quad: &Handle<Mesh>) -> impl Bundle {
        (
            shape(id),
            Mesh2d(quad.clone()),
            MeshMaterial2d::<SmudMaterial>::default(),
            NoAutoAabb,
        )
    }

    /// The first param in the slot the entity's tag names.
    fn drawn_id(world: &World, entity: Entity) -> f32 {
        let array = world.resource::<GpuComponentArray<SmudShape>>();
        let buffer = world
            .resource::<Assets<ShaderBuffer>>()
            .get(&array.buffer)
            .unwrap();
        let instances = buffer.cast_slice::<GpuSmudInstance>().unwrap();
        let tag = **world.get::<MeshTag>(entity).unwrap();
        instances[tag as usize].params[0]
    }

    fn draws(world: &World, entity: Entity) -> bool {
        world.get::<Mesh2d>(entity).is_some()
            || world.get::<MeshMaterial2d<SmudMaterial>>(entity).is_some()
    }

    /// An entity that loses its shape stops drawing, while the shape moved
    /// into its slot draws its own data from there. A sprite keeps its mesh,
    /// and a shape put back the same frame draws its new data.
    #[test]
    fn an_entity_that_loses_its_shape_stops_drawing_it() {
        let (mut app, quad) = app();
        let sprite_mesh: Handle<Mesh> = uuid_handle!("2d0c6f6e-5d0a-4a52-8a1e-6c3f9b1d7e42");
        let world = app.world_mut();
        let wall = world.spawn(drawn_shape(1., &quad)).id();
        let sprite = world.spawn(drawn_shape(2., &quad)).id();
        let put_back = world.spawn(drawn_shape(3., &quad)).id();
        let despawned = world.spawn(drawn_shape(4., &quad)).id();
        // Last, so the slots of the shapes removed above move it.
        let moved = world.spawn(drawn_shape(5., &quad)).id();
        app.update();
        assert_eq!(drawn_id(app.world(), moved), 5.);

        let world = app.world_mut();
        for entity in [wall, sprite, put_back] {
            world.entity_mut(entity).remove::<SmudShape>();
        }
        world.entity_mut(sprite).insert(Mesh2d(sprite_mesh));
        world.entity_mut(put_back).insert(shape(6.));
        app.update();

        let world = app.world();
        assert!(!draws(world, wall), "the wall still draws");
        assert!(
            world.get::<Mesh2d>(sprite).is_some(),
            "the sprite lost its mesh"
        );
        assert!(
            world.get::<MeshMaterial2d<SmudMaterial>>(sprite).is_none(),
            "the sprite's entity still draws a shape"
        );
        assert!(draws(world, put_back), "a shape put back was dropped");
        assert_eq!(drawn_id(world, put_back), 6.);
        assert_eq!(
            drawn_id(world, moved),
            5.,
            "the moved shape draws another's data"
        );

        // Its own frame: Bevy 0.20.1 panics inserting the tag of a shape
        // moved into a freed slot when that shape is despawned too.
        app.world_mut().despawn(despawned);
        app.update();
        assert_eq!(drawn_id(app.world(), moved), 5.);
        assert_eq!(drawn_id(app.world(), put_back), 6.);
    }
}
