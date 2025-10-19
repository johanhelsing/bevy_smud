use bevy::{color::palettes::css, picking::hover::PickingInteraction, prelude::*};
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin, SmudPickingPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update_colors_on_hover)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));

    // Row 1: Default bounds (each shape uses its natural size)
    commands.spawn((
        Transform::from_translation(Vec3::new(-420., 150., 0.)),
        SmudShape::from(Rectangle::new(50., 50.)).with_color(css::TOMATO),
        OriginalColor(css::TOMATO.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-300., 150., 0.)),
        SmudShape::from(Circle::new(25.)).with_color(css::CORNFLOWER_BLUE),
        OriginalColor(css::CORNFLOWER_BLUE.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-180., 150., 0.)),
        SmudShape::from(Ellipse::new(30., 30.)).with_color(css::VIOLET),
        OriginalColor(css::VIOLET.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-60., 150., 0.)),
        SmudShape::from(Annulus::new(15., 30.)).with_color(css::MAGENTA),
        OriginalColor(css::MAGENTA.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(60., 150., 0.)),
        SmudShape::from(Capsule2d::new(10., 20.)).with_color(css::LIME),
        OriginalColor(css::LIME.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(180., 150., 0.)),
        SmudShape::from(Rhombus::new(30., 30.)).with_color(css::GOLD),
        OriginalColor(css::GOLD.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(300., 150., 0.)),
        SmudShape::from(CircularSector::from_turns(35., 0.25)).with_color(css::ORANGE_RED),
        OriginalColor(css::ORANGE_RED.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(420., 150., 0.)),
        SmudShape::from(RegularPolygon::new(30., 6)).with_color(css::AQUA),
        OriginalColor(css::AQUA.into()),
    ));

    // Row 2: Rectangular bounds (showing how Rectangle and Ellipse adapt to different bounds)
    commands.spawn((
        Transform::from_translation(Vec3::new(-420., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Rectangle::new(50., 50.)).with_color(css::TOMATO)
        },
        OriginalColor(css::TOMATO.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-300., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Circle::new(25.)).with_color(css::CORNFLOWER_BLUE)
        },
        OriginalColor(css::CORNFLOWER_BLUE.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-180., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Ellipse::new(30., 30.)).with_color(css::VIOLET)
        },
        OriginalColor(css::VIOLET.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-60., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Annulus::new(15., 30.)).with_color(css::MAGENTA)
        },
        OriginalColor(css::MAGENTA.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(60., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Capsule2d::new(10., 20.)).with_color(css::LIME)
        },
        OriginalColor(css::LIME.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(180., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(Rhombus::new(30., 30.)).with_color(css::GOLD)
        },
        OriginalColor(css::GOLD.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(300., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(CircularSector::from_turns(35., 0.25)).with_color(css::ORANGE_RED)
        },
        OriginalColor(css::ORANGE_RED.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(420., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            ..SmudShape::from(RegularPolygon::new(30., 6)).with_color(css::AQUA)
        },
        OriginalColor(css::AQUA.into()),
    ));
}

// Component to store the original color for restoring after hover
#[derive(Component)]
struct OriginalColor(Color);

// System to change colors on hover
fn update_colors_on_hover(
    mut shapes: Query<(&mut SmudShape, &OriginalColor, &PickingInteraction)>,
) {
    for (mut shape, original, interaction) in &mut shapes {
        match *interaction {
            PickingInteraction::Pressed => {
                // Brighten significantly when pressed
                shape.color = Color::WHITE;
            }
            PickingInteraction::Hovered => {
                // Brighten slightly when hovered
                let linear: LinearRgba = original.0.into();
                shape.color = Color::LinearRgba(LinearRgba {
                    red: (linear.red * 1.3).min(1.0),
                    green: (linear.green * 1.3).min(1.0),
                    blue: (linear.blue * 1.3).min(1.0),
                    alpha: linear.alpha,
                });
            }
            PickingInteraction::None => {
                // Restore original color
                shape.color = original.0;
            }
        }
    }
}
