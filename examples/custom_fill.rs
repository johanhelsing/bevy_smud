use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::{prelude::*, SIMPLE_FILL_HANDLE};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // The fill takes a distance and a color and returns another color
    let sin_fill = shaders.add_fill_body("return vec4<f32>(color.rgb, sin(d));");

    commands.spawn(ShapeBundle {
        shape: SmudShape {
            color: css::TEAL.into(),
            sdf: asset_server.load("bevy.wgsl"),
            fill: sin_fill,
            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * 600.),
        shape: SmudShape {
            color: css::BLUE.into(),
            sdf: asset_server.load("bevy.wgsl"),
            fill: SIMPLE_FILL_HANDLE,
            frame: Frame::Quad(295.),
        },
        ..default()
    });

    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * -600.),
        shape: SmudShape {
            color: css::ORANGE.into(),
            sdf: asset_server.load("bevy.wgsl"),
            fill: shaders.add_fill_body(
                r"
let d_2 = abs(d - 1.) - 1.;
let a = smud::sd_fill_alpha_fwidth(d_2);
return vec4<f32>(color.rgb, a * color.a);
            ",
            ),

            frame: Frame::Quad(295.),
        },
        ..default()
    });

    // bevy_smud comes with anti-aliasing built into the standards fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
