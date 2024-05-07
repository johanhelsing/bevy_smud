use bevy::prelude::*;
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
            SmudPlugin,
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            PanCamPlugin,
            bevy_lospec::PalettePlugin,
        ))
        .add_systems(OnEnter(GameState::Running), setup)
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

    // TODO: These could be inlined with .add_sdf_body
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
        asset_server.load("gallery/star_4.wgsl"),
        asset_server.load("gallery/horseshoe.wgsl"),
        asset_server.load("gallery/blobby_cross.wgsl"),
        asset_server.load("gallery/hexagon.wgsl"),
        asset_server.load("gallery/vesica.wgsl"),
        asset_server.load("gallery/segment.wgsl"),
        asset_server.load("gallery/triangle.wgsl"),
        asset_server.load("gallery/stairs.wgsl"),
        asset_server.load("gallery/donut.wgsl"),
    ];

    let fills = [
        // asset_server.load("fills/simple.wgsl"),
        asset_server.load("fills/cubic_falloff.wgsl"),
        asset_server.load("fills/outline.wgsl"),
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

            commands.spawn((
                ShapeBundle {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        j as f32 * spacing - h as f32 * spacing / 2.,
                        0.,
                    )),
                    shape: SmudShape {
                        color,
                        // sdf_shader: shaders[index % shaders.len()].clone(),
                        sdf: shaders.choose(&mut rng).unwrap().clone(),
                        frame: Frame::Quad(50.),
                        fill: fills.choose(&mut rng).unwrap().clone(),
                    },
                    ..default()
                },
                Index(index),
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
