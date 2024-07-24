use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bevy_shape_shader = asset_server.load("star_bevy.wgsl");

    commands.spawn(ShapeBundle {
        shape: SmudShape {
            color: Color::srgb(0.36, 0.41, 0.45),
            sdf: bevy_shape_shader,
            frame: Frame::Quad(400.),
            ..default()
        },
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
