//! This example shows additive blending for shapes
//!
//! Additive blending adds the colors together, resulting in brighter areas
//! where shapes overlap. This is useful for glow effects, light sources,
//! or other luminous objects.

use bevy::color::palettes::css;
use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_shapes)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // add_sdf_expr expects a wgsl expression
    // p is the position of a fragment within the sdf shape, with 0, 0 at the center.
    // Here we are using the built-in sd_circle function, which accepts the
    // radius as a parameter.
    let circle = shaders.add_sdf_expr("smud::sd_circle(input.pos, 70.)");

    // Create overlapping circles with normal alpha blending
    commands.spawn((
        Transform::from_xyz(-150.0, 100.0, 0.0),
        SmudShape {
            color: css::RED.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Alpha, // Normal alpha blending
            ..default()
        },
        AlphaBlendedShape,
    ));

    commands.spawn((
        Transform::from_xyz(-50.0, 100.0, 0.0),
        SmudShape {
            color: css::GREEN.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Alpha, // Normal alpha blending
            ..default()
        },
        AlphaBlendedShape,
    ));

    commands.spawn((
        Transform::from_xyz(50.0, 100.0, 0.0),
        SmudShape {
            color: css::BLUE.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Alpha, // Normal alpha blending
            ..default()
        },
        AlphaBlendedShape,
    ));

    // Create overlapping circles with additive blending
    commands.spawn((
        Transform::from_xyz(-150.0, -100.0, 0.0),
        SmudShape {
            color: css::RED.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Additive, // Additive blending
            ..default()
        },
        AdditiveBlendedShape,
    ));

    commands.spawn((
        Transform::from_xyz(-50.0, -100.0, 0.0),
        SmudShape {
            color: css::GREEN.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Additive, // Additive blending
            ..default()
        },
        AdditiveBlendedShape,
    ));

    commands.spawn((
        Transform::from_xyz(50.0, -100.0, 0.0),
        SmudShape {
            color: css::BLUE.with_alpha(0.8).into(),
            sdf: circle.clone(),
            frame: Rectangle::from_length(160.),
            blend_mode: BlendMode::Additive, // Additive blending
            ..default()
        },
        AdditiveBlendedShape,
    ));

    commands.spawn(Camera2d);
}

#[derive(Component)]
struct AlphaBlendedShape;

#[derive(Component)]
struct AdditiveBlendedShape;

fn animate_shapes(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut alpha_query: Query<
        &mut Transform,
        (With<AlphaBlendedShape>, Without<AdditiveBlendedShape>),
    >,
    mut additive_query: Query<
        &mut Transform,
        (With<AdditiveBlendedShape>, Without<AlphaBlendedShape>),
    >,
) {
    if !keys.pressed(KeyCode::Space) {
        return;
    }

    let time_factor = time.elapsed_secs() * 2.0;

    // Animate alpha blended shapes
    for (i, mut transform) in alpha_query.iter_mut().enumerate() {
        let angle = time_factor + i as f32 * 2.0 * std::f32::consts::PI / 3.0;
        let radius = 50.0;
        transform.translation.x = -100.0 + angle.cos() * radius;
        transform.translation.y = 100.0 + angle.sin() * radius;
    }

    // Animate additive blended shapes
    for (i, mut transform) in additive_query.iter_mut().enumerate() {
        let angle = time_factor + i as f32 * 2.0 * std::f32::consts::PI / 3.0;
        let radius = 50.0;
        transform.translation.x = -100.0 + angle.cos() * radius;
        transform.translation.y = -100.0 + angle.sin() * radius;
    }
}
