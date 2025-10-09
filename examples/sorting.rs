use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // pupil
    commands.spawn((
        Transform::from_translation(Vec3::Z * 3.),
        SmudShape {
            color: Color::srgb(0.0, 0.0, 0.0),
            sdf: shaders.add_sdf_body("return smud::sd_circle(input.pos, 70.);"),
            frame: Rectangle::from_length(160.),
            ..default()
        },
    ));

    // iris
    commands.spawn((
        Transform::from_translation(Vec3::Z * 2.),
        SmudShape {
            color: Color::srgb(0.46, 0.42, 0.80),
            sdf: shaders.add_sdf_body("return smud::sd_circle(input.pos, 150.);"),
            frame: Rectangle::from_length(400.),
            ..default()
        },
    ));

    // sclera
    commands.spawn((
        Transform::from_translation(Vec3::Z * 1.),
        SmudShape {
            color: Color::srgb(0.83, 0.82, 0.80),
            sdf: shaders.add_sdf_body("return smud::sd_vesica(p.yx, 400., 150.);"),
            frame: Rectangle::from_length(800.),
            ..default()
        },
    ));

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
