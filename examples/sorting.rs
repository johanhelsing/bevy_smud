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

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // pupil
    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 3.),
        shape: SmudShape {
            color: Color::rgb(0.0, 0.0, 0.0),
            sdf_shader: shaders.add_sdf_body("return sd_circle(p, 70.);"),
            frame: Frame::Quad(80.),
        },
        ..Default::default()
    });

    // iris
    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 2.),
        shape: SmudShape {
            color: Color::rgb(0.46, 0.42, 0.80),
            sdf_shader: shaders.add_sdf_body("return sd_circle(p, 150.);"),
            frame: Frame::Quad(200.),
        },
        ..Default::default()
    });

    // sclera
    commands.spawn_bundle(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 1.),
        shape: SmudShape {
            color: Color::rgb(0.83, 0.82, 0.80),
            sdf_shader: shaders.add_sdf_body("return sd_vesica(p.yx, 400., 150.);"),
            frame: Frame::Quad(400.),
        },
        ..Default::default()
    });

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());
}
