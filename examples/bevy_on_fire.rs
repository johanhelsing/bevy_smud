use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SmudShape {
        color: Color::srgb(0.36, 0.41, 0.45),
        sdf: asset_server.load("bevy_on_fire.wgsl"),
        frame: Frame::Quad(400.),
        fill: asset_server.load("fills/fire.wgsl"),
        ..default()
    });

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
