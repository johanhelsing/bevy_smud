use bevy::color::palettes::css;
use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn((Camera2d, Msaa::Off));

    // Using the ergonomic From<Rectangle> API with builder methods
    commands.spawn((
        Transform::from_translation(Vec3::new(-200., 100., 0.)),
        SmudShape::from(Rectangle::new(100., 50.))
            .with_color(css::TOMATO)
    ));

    // Another rectangle with different size and color
    commands.spawn((
        Transform::from_translation(Vec3::new(100., 100., 0.)),
        SmudShape::from(Rectangle::new(80., 80.))
            .with_color(css::CORNFLOWER_BLUE)
    ));

    // Using struct initialization with spread operator
    commands.spawn((
        Transform::from_translation(Vec3::new(-200., -100., 0.)),
        SmudShape {
            color: css::LIMEGREEN.into(),
            ..SmudShape::from(Rectangle::new(120., 40.))
        }
    ));

    // Rotated rectangle
    commands.spawn((
        Transform::from_translation(Vec3::new(100., -100., 0.))
            .with_rotation(Quat::from_rotation_z(0.5)),
        SmudShape::from(Rectangle::new(100., 50.))
            .with_color(css::ORANGE)
    ));

    // Scaled rectangle
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 0., 0.))
            .with_scale(Vec3::splat(1.5)),
        SmudShape::from(Rectangle::new(60., 60.))
            .with_color(css::HOT_PINK)
    ));

    // Rectangle with simple fill
    commands.spawn((
        Transform::from_translation(Vec3::new(250., 0., 0.)),
        SmudShape::from(Rectangle::new(50., 100.))
            .with_color(css::YELLOW)
            .with_fill(SIMPLE_FILL_HANDLE)
    ));
}
