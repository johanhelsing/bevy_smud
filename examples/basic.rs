use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    // add_sdf_expr expects a wgsl expression
    // p is the position of a fragment within the sdf shape, with 0, 0 at the center.
    // Here we are using the built-in sd_circle function, which accepts the
    // radius as a parameter.
    let circle = shaders.add_sdf_expr("sd_circle(p, 70.)");

    // There are other ways to define sdfs as well:
    // .add_sdf_body let's you add multiple lines and needs to end with a return statements
    let peanut = shaders.add_sdf_body(
        r"
// Taking the absolute value of p.x creates a vertical line of symmetry
let p = vec2<f32>(abs(p.x), p.y);
// By subtracting from p, we can move shapes
return sd_circle(p - vec2<f32>(20., 0.), 40.);
    ",
    );

    // If the sdf gets very complicated, you can keep it in a .wgsl file:
    let bevy = asset_server.load("bevy.wgsl");

    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            color: Color::TOMATO,
            sdf: circle,
            // The frame needs to be bigger than the shape we're drawing
            // Since the circle has radius 70, we make the half-size of the quad 80.
            frame: Frame::Quad(80.),
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * 200.),
        shape: SmudShape {
            color: Color::rgb(0.7, 0.6, 0.4),
            sdf: peanut,
            frame: Frame::Quad(80.),
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(ShapeBundle {
        transform: Transform {
            translation: Vec3::X * -200.,
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

    commands.spawn_bundle(Camera2dBundle::default());
}
