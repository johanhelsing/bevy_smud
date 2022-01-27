use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_pancam::*;
use bevy_so_smooth::*;
use rand::prelude::*;

fn main() {
    let mut app = App::new();

    AssetLoader::new(GameState::Loading)
        .continue_to_state(GameState::Running)
        .with_collection::<AssetHandles>()
        .build(&mut app);

    #[cfg(feature = "smud_shader_hot_reloading")]
    app.insert_resource(bevy::asset::AssetServerSettings {
        watch_for_changes: true,
        ..Default::default()
    });

    app.add_state(GameState::Loading)
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SoSmoothPlugin)
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(PanCamPlugin)
        .add_plugin(bevy_lospec::PalettePlugin)
        .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup))
        // .add_system_set(SystemSet::on_update(GameState::Running).with_system(update))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Running,
}

#[derive(AssetCollection)]
struct AssetHandles {
    #[asset(path = "vinik24.json")]
    palette: Handle<bevy_lospec::Palette>,
}

#[derive(Component)]
struct Index(usize);

fn setup(
    mut commands: Commands,
    assets: Res<AssetHandles>,
    palettes: Res<Assets<bevy_lospec::Palette>>,
    asset_server: Res<AssetServer>,
) {
    let palette = palettes.get(assets.palette.clone()).unwrap();
    let mut rng = rand::thread_rng();
    let spacing = 100.0;
    // let w = 316;
    let w = 200;
    // let w = 420;
    // let w = 10;
    // let w = 80;
    let h = w;
    info!("Adding {} shapes", w * h);

    let clear_color = palette.lightest();
    // let clear_color = palette.darkest();
    commands.insert_resource(ClearColor(clear_color));

    let shaders = vec![
        asset_server.load("gallery/box.wgsl"),
        asset_server.load("gallery/circle.wgsl"),
        asset_server.load("gallery/heart.wgsl"),
        asset_server.load("gallery/moon.wgsl"),
        asset_server.load("gallery/pie.wgsl"),
        asset_server.load("gallery/egg.wgsl"),
        asset_server.load("gallery/rounded_x.wgsl"),
        asset_server.load("gallery/ellipse.wgsl"),
        asset_server.load("gallery/star_5.wgsl"),
        asset_server.load("gallery/horseshoe.wgsl"),
        asset_server.load("gallery/blobby_cross.wgsl"),
        asset_server.load("gallery/hexagon.wgsl"),
        asset_server.load("gallery/vesica.wgsl"),
        asset_server.load("gallery/segment.wgsl"),
        asset_server.load("gallery/triangle.wgsl"),
        asset_server.load("gallery/stairs.wgsl"),
        asset_server.load("gallery/donut.wgsl"),
    ];

    for i in 0..w {
        for j in 0..h {
            let color = palette
                .iter()
                .filter(|c| *c != &clear_color)
                .choose(&mut rng)
                .copied()
                .unwrap_or(Color::PINK);

            let index = i + j * w;

            commands
                .spawn_bundle(ShapeBundle {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        j as f32 * spacing - h as f32 * spacing / 2.,
                        0.,
                    )),
                    shape: SmudShape {
                        color,
                        // sdf_shader: shaders[index % shaders.len()].clone(),
                        sdf_shader: shaders.choose(&mut rng).unwrap().clone(),
                        frame: Frame::Quad(50.),
                    },
                    ..Default::default()
                })
                .insert(Index(index));
        }
    }

    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    // camera_bundle.orthographic_projection.scale = 1. / 10.;
    commands
        .spawn_bundle(camera_bundle)
        .insert(PanCam::default());
}

// fn update(mut query: Query<(&mut Transform, &Index), With<SmudShape>>, time: Res<Time>) {
//     let t = time.time_since_startup().as_secs_f64();

//     for (mut tx, index) in query.iter_mut() {
//         let s = f64::sin(t + (index.0 as f64) / 1.0) as f32;
//         tx.scale = Vec3::splat(s);
//     }
// }
