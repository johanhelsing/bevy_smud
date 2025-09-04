use bevy::prelude::*;
use bevy_pancam::PanCam;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save
    let bevy = asset_server.load("bevy.wgsl");

    commands.spawn((
        Transform {
            scale: Vec3::splat(0.4),
            ..default()
        },
        SmudShape {
            color: Color::WHITE,
            sdf: bevy,
            // You can also specify a custom type of fill
            // The simple fill is just a simple anti-aliased opaque fill
            fill: SIMPLE_FILL_HANDLE,
            frame: Frame::Quad(295.),
            ..default()
        },
    ));

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
