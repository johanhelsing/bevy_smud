use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_smud::*;

/// This example just shows that transforms work

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
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
    };

    let shape = SmudShape {
        color: Color::rgb(0.36, 0.41, 0.45),
        sdf: bevy_shape_shader,
        frame: Frame::Quad(295.),
        ..default()
    };

    // Bevies, all the way down
    commands
        .spawn(ShapeBundle {
            shape: shape.clone(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ShapeBundle {
                    transform,
                    shape: shape.clone(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(ShapeBundle {
                        transform,
                        shape: shape.clone(),
                        ..default()
                    });
                });
        });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
