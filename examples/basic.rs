use bevy::{
    asset::AssetServerSettings, prelude::*, render::render_resource::PrimitiveTopology,
    sprite::Mesh2dHandle,
};
use bevy_so_smooth::*;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SoSmoothPlugin)
        // .add_startup_system(setup)
        .add_startup_system(star)
        .run();
}

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<CustomMaterial>>,
// ) {
//     let size = Vec2::new(300.0, 300.0);

//     let material = materials.add(CustomMaterial {
//         color: Vec4::new(0.05, 0.05, 0.1, 1.0),
//     });

//     commands.spawn().insert_bundle(MaterialMesh2dBundle {
//         mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(size)))),
//         material,
//         ..Default::default()
//     });

//     commands.spawn_bundle(OrthographicCameraBundle::new_2d());
// }

fn star(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Let's define the mesh for the object we want to draw: a nice quad.
    let mut quad = Mesh::new(PrimitiveTopology::TriangleStrip);
    let w = 100.;
    let v_pos = vec![[-w, -w], [w, -w], [-w, w], [w, w]];
    // Set the position attribute
    quad.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    // And a RGB color attribute as well
    let v_color = vec![[0.5, 0.3, 0.1, 1.0]; 4];
    quad.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);
    // let indices = vec![0, 1, 2, 3];
    // quad.set_indices(Some(Indices::U32(indices)));

    // We can now spawn the entities for the star and the camera
    commands.spawn_bundle((
        // We use a marker component to identify the custom colored meshes
        SmudShape::default(),
        // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
        Mesh2dHandle(meshes.add(quad)),
        // These other components are needed for 2d meshes to be rendered
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
    commands
        // And use an orthographic projection
        .spawn_bundle(OrthographicCameraBundle::new_2d());
}
