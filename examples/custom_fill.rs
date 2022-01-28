use bevy::prelude::*;
use bevy_pancam::*;
use bevy_so_smooth::prelude::*;

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
        .add_plugin(SoSmoothPlugin)
        .add_plugin(PanCamPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // The fill takes a distance and a color and returns another color
    let sin_fill = shaders.add_fill_body("return vec4<f32>(color.rgb, sin(d));");

    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            color: Color::TEAL,
            sdf_shader: asset_server.load("bevy.wgsl"),
            fill_shader: sin_fill,
            frame: Frame::Quad(295.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());
}
