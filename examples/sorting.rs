use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // pupil
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 3.),
        shape: SmudShape {
            color: Color::rgb(0.0, 0.0, 0.0),
            sdf: shaders.add_sdf_body("return shapes::sd_circle(p, 70.);"),
            frame: Frame::Quad(80.),
            ..default()
        },
        ..default()
    });

    // iris
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 2.),
        shape: SmudShape {
            color: Color::rgb(0.46, 0.42, 0.80),
            sdf: shaders.add_sdf_body("return shapes::sd_circle(p, 150.);"),
            frame: Frame::Quad(200.),
            ..default()
        },
        ..default()
    });

    // sclera
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 1.),
        shape: SmudShape {
            color: Color::rgb(0.83, 0.82, 0.80),
            sdf: shaders.add_sdf_body("return shapes::sd_vesica(p.yx, 400., 150.);"),
            frame: Frame::Quad(400.),
            ..default()
        },
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
