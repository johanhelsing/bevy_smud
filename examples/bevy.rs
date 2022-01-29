use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "smud_shader_hot_reloading")]
    app.insert_resource(bevy::asset::AssetServerSettings {
        watch_for_changes: true,
        ..Default::default()
    });

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_plugin(PanCamPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bevy_shape_shader = asset_server.load("bevy.wgsl");

    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            color: Color::rgb(0.36, 0.41, 0.45),
            sdf_shader: bevy_shape_shader,
            frame: Frame::Quad(400.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());
}
