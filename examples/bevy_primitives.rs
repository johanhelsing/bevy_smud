use bevy::{color::palettes::css, picking::hover::PickingInteraction, prelude::*};
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin, SmudPickingPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_bounds, update_colors_on_hover))
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

    // Row 2: Animated bounds (shapes smoothly transition between tall and wide)
    commands.spawn((
        Transform::from_translation(Vec3::new(-420., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::TOMATO.into(),
            ..SmudShape::from(Rectangle::new(50., 50.))
        },
        OriginalColor(css::TOMATO.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-300., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::CORNFLOWER_BLUE.into(),
            ..SmudShape::from(Circle::new(25.))
        },
        OriginalColor(css::CORNFLOWER_BLUE.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-180., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::VIOLET.into(),
            ..SmudShape::from(Ellipse::new(30., 30.))
        },
        OriginalColor(css::VIOLET.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-60., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::MAGENTA.into(),
            ..SmudShape::from(Annulus::new(15., 30.))
        },
        OriginalColor(css::MAGENTA.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(60., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::LIME.into(),
            ..SmudShape::from(Capsule2d::new(10., 20.))
        },
        OriginalColor(css::LIME.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(180., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::GOLD.into(),
            ..SmudShape::from(Rhombus::new(30., 30.))
        },
        OriginalColor(css::GOLD.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(300., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::ORANGE_RED.into(),
            ..SmudShape::from(CircularSector::from_turns(35., 0.25))
        },
        OriginalColor(css::ORANGE_RED.into()),
        AnimatedBounds,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(420., -150., 0.)),
        SmudShape {
            bounds: Rectangle::new(70., 40.),
            color: css::AQUA.into(),
            ..SmudShape::from(RegularPolygon::new(30., 6))
        },
        OriginalColor(css::AQUA.into()),
        AnimatedBounds,
    ));
}

// Component to store the original color for restoring after hover
#[derive(Component)]
struct OriginalColor(Color);

// Marker component for shapes with animated bounds
#[derive(Component)]
struct AnimatedBounds;

// System to animate bounds based on time
fn animate_bounds(time: Res<Time>, mut shapes: Query<&mut SmudShape, With<AnimatedBounds>>) {
    let t = time.elapsed_secs();

    // Use sine and cosine to smoothly transition between tall and wide
    // sin ranges from -1 to 1, we'll map it to create width variation
    // cos ranges from -1 to 1, we'll map it to create height variation
    let width = 55.0 + 15.0 * t.sin(); // Ranges from 40 to 70
    let height = 55.0 - 15.0 * t.cos(); // Ranges from 40 to 70, offset from width

    for mut shape in &mut shapes {
        shape.bounds = Rectangle::new(width, height);
    }
}

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
