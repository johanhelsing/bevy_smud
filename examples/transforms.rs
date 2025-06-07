//! This example just shows that transforms work

use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_smud::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
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
        color: Color::srgb(0.36, 0.41, 0.45),
        sdf: bevy_shape_shader,
        frame: Frame::Quad(295.),
        ..default()
    };

    // Bevies, all the way down
    commands.spawn(shape.clone()).with_children(|parent| {
        parent
            .spawn((transform, shape.clone()))
            .with_children(|parent| {
                parent.spawn((transform, shape.clone()));
            });
    });

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
