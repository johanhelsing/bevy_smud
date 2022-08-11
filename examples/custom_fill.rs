use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::{prelude::*, SIMPLE_FILL_HANDLE};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
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
            sdf: asset_server.load("bevy.wgsl"),
            fill: sin_fill,
            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * 600.),
        shape: SmudShape {
            color: Color::BLUE,
            sdf: asset_server.load("bevy.wgsl"),
            fill: SIMPLE_FILL_HANDLE.typed(),
            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * -600.),
        shape: SmudShape {
            color: Color::ORANGE,
            sdf: asset_server.load("bevy.wgsl"),
            fill: shaders.add_fill_body(
                r"
let d_2 = abs(d - 1.) - 1.;
let a = sd_fill_alpha_fwidth(d_2);
return vec4<f32>(color.rgb, a * color.a);
            ",
            ),

            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam::default());
}
