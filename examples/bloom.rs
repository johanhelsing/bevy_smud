//! This example shows that bevy_smud works with bloom enabled
//!
//! Note that you could probably achieve chieper and higher quality bloom-like
//! effects by creating a custom fill.

use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // add_sdf_expr expects a wgsl expression
    // p is the position of a fragment within the sdf shape, with 0, 0 at the center.
    // Here we are using the built-in sd_circle function, which accepts the
    // radius as a parameter.
    let circle = shaders.add_sdf_expr("sd_circle(p, 70.)");

    commands.spawn(ShapeBundle {
        shape: SmudShape {
            color: Color::TOMATO,
            sdf: circle,
            // The frame needs to be bigger than the shape we're drawing
            // Since the circle has radius 70, we make the half-size of the quad 80.
            frame: Frame::Quad(80.),
            fill: SIMPLE_FILL_HANDLE.typed(),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings {
            threshold: 0.2,
            ..default()
        },
    ));
}
