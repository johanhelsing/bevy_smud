use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_pancam::*;
use bevy_smud::*;
use rand::prelude::*;

fn main() {
    App::new()
        .init_state::<GameState>()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Running)
                .load_collection::<AssetHandles>(),
        )
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            SmudPlugin,
            PanCamPlugin,
            bevy_lospec::PalettePlugin,
        ))
        .add_systems(OnEnter(GameState::Running), setup)
        // .add_system_set(SystemSet::on_update(GameState::Running).with_system(update))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, States, Default)]
enum GameState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource, AssetCollection)]
struct AssetHandles {
    #[asset(path = "vinik24.json")]
    palette: Handle<bevy_lospec::Palette>,
}

#[allow(dead_code)]
#[derive(Component)]
struct Index(usize);

fn setup(
    mut commands: Commands,
    assets: Res<AssetHandles>,
    palettes: Res<Assets<bevy_lospec::Palette>>,
    asset_server: Res<AssetServer>,
) {
    let palette = palettes.get(&assets.palette).unwrap();
    let mut rng = rand::thread_rng();
    let spacing = 800.0;
    let w = 316;
    // let w = 420;
    // let w = 10;
    let h = w;
    info!("Adding {} shapes", w * h);

    let clear_color = palette.lightest();
    commands.insert_resource(ClearColor(clear_color));

    let bevy_shape_shader = asset_server.load("bevy.wgsl");

    for i in 0..w {
        for j in 0..h {
            let color = palette
                .iter()
                .filter(|c| *c != &clear_color)
                .choose(&mut rng)
                .copied()
                .unwrap_or(Color::PINK);

            commands.spawn((
                ShapeBundle {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        j as f32 * spacing - h as f32 * spacing / 2.,
                        0.,
                    )),
                    shape: SmudShape {
                        color,
                        sdf: bevy_shape_shader.clone(),
                        frame: Frame::Quad(295.),
                        ..default()
                    },
                    ..default()
                },
                Index(i + j * w),
            ));
        }
    }
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

// fn update(mut query: Query<(&mut Transform, &Index), With<SmudShape>>, time: Res<Time>) {
//     let t = time.time_since_startup().as_secs_f64();

//     for (mut tx, index) in query.iter_mut() {
//         let s = f64::sin(t + (index.0 as f64) / 1.0) as f32;
//         tx.scale = Vec3::splat(s);
//     }
// }
