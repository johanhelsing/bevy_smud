//! Render layers test.
//!
//! Split-screen with two cameras on different render layers.
//! Red circle on layer 0 (left), blue circle on layer 1 (right).
//! Each should only appear on its own side.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    let circle = shaders.add_sdf_expr("smud::sd_circle(p, 70.)");

    // Camera A: left half of the screen, layer 0
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            viewport: Some(bevy::camera::Viewport {
                physical_position: UVec2::ZERO,
                physical_size: UVec2::new(512, 720),
                ..default()
            }),
            ..default()
        },
        Name::new("camera_a"),
    ));

    // Camera B: right half of the screen, layer 1
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            viewport: Some(bevy::camera::Viewport {
                physical_position: UVec2::new(512, 0),
                physical_size: UVec2::new(512, 720),
                ..default()
            }),
            ..default()
        },
        RenderLayers::layer(1),
        Name::new("camera_b"),
    ));

    // Red circle on layer 0 - should only appear on camera A (left half)
    commands.spawn((
        SmudShape {
            color: Color::srgb(1.0, 0.2, 0.2),
            sdf: circle.clone(),
            bounds: Rectangle::from_length(160.),
            ..default()
        },
        Transform::from_xyz(-80., 0., 0.),
    ));

    // Blue circle on layer 1 - should only appear on camera B (right half)
    commands.spawn((
        SmudShape {
            color: Color::srgb(0.2, 0.2, 1.0),
            sdf: circle,
            bounds: Rectangle::from_length(160.),
            ..default()
        },
        Transform::from_xyz(80., 0., 0.),
        RenderLayers::layer(1),
    ));
}
