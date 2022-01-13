use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_so_smooth::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SoSmoothPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let size = Vec2::new(300.0, 300.0);

    let material = materials.add(CustomMaterial {
        color: Vec4::new(0.05, 0.05, 0.1, 1.0),
    });

    commands.spawn().insert_bundle(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(size)))),
        material,
        ..Default::default()
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
