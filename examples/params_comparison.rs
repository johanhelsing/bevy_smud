use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // OLD WAY: Without params - these shapes work without any boilerplate
    let circle = shaders.add_sdf_expr("smud::sd_circle(p, 50.)");
    let heart = shaders.add_sdf_expr("smud::sd_heart(p)");
    
    // NEW WAY: With params - these shapes accept parameters
    let rect = shaders.add_sdf_expr_with_params("smud::sd_box(p, params.xy)");
    let star = shaders.add_sdf_expr_with_params("smud::sd_star(p, 5, params.x, params.y)");

    // Spawn shapes without params - clean and simple!
    commands.spawn((
        Transform::from_translation(Vec3::new(-300., 100., 0.)),
        SmudShape {
            color: css::TOMATO.into(),
            sdf: circle,
            frame: Frame::Quad(60.),
            ..default() // params defaults to None - no boilerplate!
        },
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(-100., 100., 0.)),
        SmudShape {
            color: css::PINK.into(),
            sdf: heart,
            frame: Frame::Quad(80.),
            ..default() // again, no params needed!
        },
    ));

    // Spawn shapes with params - explicit when needed
    commands.spawn((
        Transform::from_translation(Vec3::new(100., 100., 0.)),
        SmudShape {
            color: css::LIGHT_BLUE.into(),
            sdf: rect,
            frame: Frame::Quad(80.),
            params: Some(Vec4::new(60., 40., 0., 0.)), // width=60, height=40
            ..default()
        },
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(300., 100., 0.)),
        SmudShape {
            color: css::GOLD.into(),
            sdf: star,
            frame: Frame::Quad(80.),
            params: Some(Vec4::new(30., 10., 0., 0.)), // outer_radius=30, inner_radius=10
            ..default()
        },
    ));

    commands.spawn(Camera2d);
}
