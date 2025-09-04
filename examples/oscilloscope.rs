use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::prelude::*;

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
    commands.spawn(SmudShape {
        color: css::ORANGE.into(),
        sdf: asset_server.load("bevy.wgsl"),
        fill: shaders.add_fill_body(
                r"
var col = color.rgb / sqrt(abs(d));
// col *= smoothstep(1.5, 0.5, length(p)); // We don't have p. This would give a vignette.

// This is a brighter effect.
return vec4<f32>(col, color.a);
// This is a darker effect.
// return vec4<f32>(aces_approx(col), color.a);
}

// HACK: We're gonna cheat on this template and add an auxiliary function.

// License: Unknown, author: Matt Taylor (https://github.com/64), found: https://64.github.io/tonemapping/
fn aces_approx(v_: vec3<f32>) -> vec3<f32> {
    var v = max(v_, vec3<f32>(0.0));
    v *= 0.6;
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    return saturate((v * (a * v + b)) / (v * (c * v + d) + e));
",
        ),
        frame: Frame::Quad(295.),
        ..default()
    });

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off));
}
