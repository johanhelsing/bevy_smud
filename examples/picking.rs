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

    // Spawn a single SDF shape (a circle) with Pickable
    commands.spawn((
        SmudShape {
            color: css::ORANGE.into(),
            sdf: shaders.add_sdf_body("return smud::sd_circle(p, 100.);"),
            frame: Frame::Quad(105.), // A little larger than the circle
            ..default()
        },
        Pickable::default(), // This will automatically add PickingInteraction
    ));

    info!("Hover over the circle to see it change color! Click to see the pressed state.");
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
