use bevy::{prelude::*, render::view::Hdr};
use bevy_pancam::*;
use bevy_smud::*;

#[allow(unused)]
#[derive(Resource)]
struct Shaders(Vec<Handle<Shader>>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins((DefaultPlugins, SmudPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update_mouse_position)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // bevy_on_fire.wgsl uses functions from bevy.wgsl, so we need to make sure bevy.wgsl is loaded
    commands.insert_resource(Shaders(vec![asset_server.load("bevy.wgsl")]));

    commands.spawn((
        SmudShape {
            color: Color::srgb(0.36, 0.41, 0.45),
            sdf: asset_server.load("bevy_on_fire.wgsl"),
            bounds: Rectangle::from_length(800.),
            fill: asset_server.load("fills/fire.wgsl"),
            ..default()
        },
        BevyShape,
    ));

    // bevy_smud comes with anti-aliasing built into the standard fills
    // which is more efficient than MSAA, and also works on Linux, wayland
    commands.spawn((Camera2d, PanCam::default(), Msaa::Off, Hdr));
}

#[derive(Component)]
struct BevyShape;

fn update_mouse_position(
    shape: Single<(&mut SmudShape, &GlobalTransform), With<BevyShape>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) -> Result {
    let (mut shape, shape_transform) = shape.into_inner();

    let Some(cursor_pos) = window.cursor_position() else {
        // Outside the window
        return Ok(());
    };

    let (camera, camera_transform) = camera.into_inner();

    // Convert screen coordinates to world coordinates
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos)?;

    // Convert world coordinates to local coordinates (relative to shape)
    let local_pos = shape_transform
        .affine()
        .inverse()
        .transform_point3(world_pos.extend(0.0));

    // Pass mouse position to shader via shape params
    shape.params = Vec4::new(local_pos.x, local_pos.y, 0.0, 0.0);

    Ok(())
}
