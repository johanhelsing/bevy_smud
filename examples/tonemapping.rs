//! This example shows that bevy_smud works with tonampping
//!
//! There are two shapes rendered:
//!
//! - A circle, rendered by bevy_smud
//! - A squaree rendered by bevy_sprite
//!
//! If tonemapping is working correctly, they should have the same color

use bevy::color::palettes::css;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    let circle = shaders.add_sdf_expr("smud::sd_circle(input.pos, 70.)");

    commands.spawn((
        Transform::from_xyz(100., 0., 0.),
        SmudShape {
            color: css::TOMATO.into(),
            sdf: circle,
            frame: Frame::quad_half_size(80.),
            fill: SIMPLE_FILL_HANDLE,
            ..default()
        },
    ));

    // bevy square for comparison
    commands.spawn((
        Transform::from_xyz(-100., 0., 0.),
        Sprite {
            color: css::TOMATO.into(),
            custom_size: Some(Vec2::splat(160.)),
            ..default()
        },
    ));

    commands.spawn((
        Camera2d,
        // bevy_smud comes with anti-aliasing built into the standard fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        Msaa::Off,
        // Reinhard tonemapping looks pretty different from no tonemapping,
        // so we can clearly see the difference between tonemapping and no tonemapping
        Tonemapping::Reinhard,
    ));
}
