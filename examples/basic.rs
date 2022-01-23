use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_so_smooth::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "smud_shader_hot_reloading")]
    app.insert_resource(bevy::asset::AssetServerSettings {
        watch_for_changes: true,
        ..Default::default()
    });

    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(SoSmoothPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let spacing = 30.0;
    let w = 20;
    let h = w;
    for i in 0..w {
        for j in 0..h {
            commands.spawn_bundle(ShapeBundle {
                transform: Transform::from_translation(Vec3::new(
                    i as f32 * spacing - w as f32 * spacing / 2.,
                    j as f32 * spacing - h as f32 * spacing / 2.,
                    0.,
                )),
                shape: SmudShape::Arc(1.),
                ..Default::default()
            });
            commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        }
    }
}
