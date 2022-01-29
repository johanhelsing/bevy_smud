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

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // add_sdf_body expects a wgsl function body
    // p is the position of a fragment within the sdf shape, with 0, 0 at the center.
    // Here we are using the built-in sd_circle function, which accepts the
    // radius as a parameter.
    let circle = shaders.add_sdf_body("return sd_circle(p, 70.);");

    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            color: Color::OLIVE,
            sdf: circle,
            // The frame needs to be bigger than the shape we're drawing
            // Since the circle has radius 70, we make the half-size of the quad 80.
            frame: Frame::Quad(80.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
