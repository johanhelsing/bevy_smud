use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_smud::*;

/// This example just shows that transforms work

fn main() {
    let mut app = App::new();

    #[cfg(feature = "smud_shader_hot_reloading")]
    app.insert_resource(bevy::asset::AssetServerSettings {
        watch_for_changes: true,
        ..Default::default()
    });

    app.insert_resource(Msaa { samples: 4 })
        // .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_plugin(PanCamPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bevy_shape_shader = asset_server.load("bevy.wgsl");

    let transform = Transform {
        scale: Vec3::splat(0.05),
        translation: Vec3::new(62., 137., 0.),
        rotation: Quat::from_rotation_z(1.0),
        // rotation: Quat::IDENTITY,
    };

    let shape = SmudShape {
        color: Color::rgb(0.36, 0.41, 0.45),
        sdf_shader: bevy_shape_shader.clone(),
        frame: Frame::Quad(295.),
    };

    // Bevies, all the way down
    commands
        .spawn_bundle(ShapeBundle {
            shape: shape.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ShapeBundle {
                    transform,
                    shape: shape.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ShapeBundle {
                        transform,
                        shape: shape.clone(),
                        ..Default::default()
                    });
                });
        });

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());
}
