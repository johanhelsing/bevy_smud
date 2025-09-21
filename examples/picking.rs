use bevy::color::palettes::css;
use bevy::picking::{hover::PickingInteraction, prelude::*};
use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin, SmudPickingPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update_hover_colors)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    commands.spawn(Camera2d);

    // Left circle: Uses precise SDF-based picking
    commands.spawn((
        Transform::from_translation(Vec3::new(-350.0, 0.0, 0.0)),
        SmudShape {
            color: css::ORANGE.into(),
            sdf: shaders.add_sdf_expr("smud::sd_circle(p, 100.)"),
            frame: Frame::Quad(150.), // Frame is larger than the circle
            ..default()
        },
        Pickable::default(),
        SmudPickingShape::new(|p| {
            // Circle SDF using the sdf module
            sdf::circle(p, 100.0)
        }),
    ));

    // Right star: Uses frame-based picking (no SdfPickingShape component)
    commands.spawn((
        Transform::from_translation(Vec3::new(350.0, 0.0, 0.0)),
        SmudShape {
            color: css::ORANGE.into(),
            sdf: shaders.add_sdf_expr("smud::sd_star_5_(p, 60.0, 2.0)"),
            frame: Frame::Quad(150.), // Frame is larger than the star
            ..default()
        },
        Pickable::default(),
    ));

    // Center heart: Uses precise SDF-based picking
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        SmudShape {
            color: css::ORANGE.into(),
            sdf: shaders.add_sdf_expr("smud::sd_heart((p / 160.0) - vec2(0.0, -0.5)) * 160.0"),
            frame: Frame::Quad(150.), // Frame is larger than the heart
            ..default()
        },
        Pickable::default(),
        SmudPickingShape::new(|p| {
            // Heart SDF using the sdf module, scaled and offset
            sdf::heart((p / 160.0) - Vec2::new(0.0, -0.5)) * 160.0
        }),
    ));

    info!("Left circle: Precise SDF-based picking - only responds inside the circle");
    info!("Center heart: Precise SDF-based picking - only responds inside the heart");
    info!("Right star: Frame-based picking - responds in the entire square frame");
    info!("Hover and click to see the difference!");
}

fn update_hover_colors(
    mut query: Query<(&mut SmudShape, &PickingInteraction), Changed<PickingInteraction>>,
) {
    for (mut shape, interaction) in query.iter_mut() {
        shape.color = match interaction {
            PickingInteraction::Hovered => css::TOMATO.into(),
            PickingInteraction::None => css::ORANGE.into(),
            PickingInteraction::Pressed => css::DARK_MAGENTA.into(),
        };
    }
}
