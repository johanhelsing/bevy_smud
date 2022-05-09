use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save
    let bevy = asset_server.load("bevy.wgsl");

    commands.spawn_bundle(ShapeBundle {
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

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
