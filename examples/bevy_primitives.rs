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
    // Spawn a camera
    commands.spawn((Camera2d, Msaa::Off));

    commands.spawn((
        Transform::from_translation(Vec3::new(-200., 100., 0.)),
        SmudShape::from(Rectangle::new(100., 50.)).with_color(css::TOMATO),
        OriginalColor(css::TOMATO.into()),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(100., 100., 0.)),
        SmudShape::from(Circle::new(40.)).with_color(css::CORNFLOWER_BLUE),
        OriginalColor(css::CORNFLOWER_BLUE.into()),
    ));

    // Using struct initialization with spread operator
    commands.spawn((
        Transform::from_translation(Vec3::new(-200., -100., 0.)),
        SmudShape {
            color: css::LIMEGREEN.into(),
            ..SmudShape::from(Rectangle::new(120., 40.))
        },
    ));

    // Rotated rectangle
    commands.spawn((
        Transform::from_translation(Vec3::new(100., -100., 0.))
            .with_rotation(Quat::from_rotation_z(0.5)),
        SmudShape::from(Rectangle::new(100., 50.)).with_color(css::ORANGE),
    ));

    // Scaled rectangle
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 0., 0.)).with_scale(Vec3::splat(1.5)),
        SmudShape::from(Rectangle::new(60., 60.)).with_color(css::HOT_PINK),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(250., 0., 0.)),
        SmudShape::from(Rectangle::new(50., 100.))
            .with_color(css::YELLOW)
            .with_fill(SIMPLE_FILL_HANDLE),
        OriginalColor(css::YELLOW.into()),
    ));

    // Ellipse
    commands.spawn((
        Transform::from_translation(Vec3::new(-250., 0., 0.)),
        SmudShape::from(Ellipse::new(70., 40.)).with_color(css::VIOLET),
        OriginalColor(css::VIOLET.into()),
    ));

    // Rotated ellipse
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 150., 0.))
            .with_rotation(Quat::from_rotation_z(0.8)),
        SmudShape::from(Ellipse::new(80., 30.)).with_color(css::TURQUOISE),
        OriginalColor(css::TURQUOISE.into()),
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
