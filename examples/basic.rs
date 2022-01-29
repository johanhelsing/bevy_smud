use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_smud::*;

// struct BevyShape;

// TODO: check if I can just use handles render assets instead?
// impl SdfShapeShader for BevyShape {
//     fn shader(asset_server: &AssetServer) -> Handle<Shader> {
//         asset_server.load("bevy.wgsl")
//     }
// }

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // .add_plugin(SdfShapePlugin::<BevyShape>::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(SmudPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    let bevy_shader = Shader::from_wgsl(include_str!("../assets/bevy.wgsl"));
    let bevy_shape_shader = shaders.add(bevy_shader.into());

    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            sdf_shader: Some(bevy_shape_shader),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
