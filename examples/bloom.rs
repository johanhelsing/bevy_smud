//! This example shows that bevy_smud works with bloom enabled
//!
//! Note that you could probably achieve cheaper and higher quality bloom-like
//! effects by creating a custom fill.

use bevy::color::palettes::css;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // add_sdf_expr expects a wgsl expression
    // p is the position of a fragment within the sdf shape, with 0, 0 at the center.
    // Here we are using the built-in sd_circle function, which accepts the
    // radius as a parameter.
    let circle = shaders.add_sdf_expr("smud::sd_circle(input.pos, 70.)");

    commands.spawn(SmudShape {
        color: css::TOMATO.into(),
        sdf: circle,
        // The bounds need to be bigger than the shape we're drawing
        // Since the circle has radius 70, we make the bounds 160 (with some padding).
        bounds: Rectangle::from_length(160.),
        fill: SIMPLE_FILL_HANDLE,
        ..default()
    });

    commands.spawn((
        Camera2d,
        // bevy_smud comes with anti-aliasing built into the standard fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        Msaa::Off,
        Hdr,
        Bloom {
            intensity: 0.7,
            ..default()
        },
    ));
}
