use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // enable hot-reloading so we can see changes to wgsl files without relaunching the app
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(SmudPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save
    let bevy = asset_server.load("bevy.wgsl");

    commands.spawn(ShapeBundle {
        transform: Transform {
            scale: Vec3::splat(0.4),
            ..default()
        },
        shape: SmudShape {
            color: Color::WHITE,
            sdf: bevy,
            // You can also specify a custom type of fill
            // The simple fill is just a simple anti-aliased opaque fill
            fill: SIMPLE_FILL_HANDLE.typed(),
            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
