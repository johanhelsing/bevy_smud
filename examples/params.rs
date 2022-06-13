use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_smud::prelude::*;
use rand::{prelude::IteratorRandom, random};

// this example shows how to use per-instance parameters in shapes
// in this simple example, a width and height is passed to a box shape,
// but it could be used for almost anything.

fn main() {
    let mut app = App::new();

    AssetLoader::new(GameState::Loading)
        .continue_to_state(GameState::Running)
        .with_collection::<AssetHandles>()
        .build(&mut app);

    app.add_state(GameState::Loading)
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_plugin(bevy_lospec::PalettePlugin)
        .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup))
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

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    assets: Res<AssetHandles>,
    palettes: Res<Assets<bevy_lospec::Palette>>,
) {
    let box_sdf = shaders.add_sdf_expr("sd_box(p, params.xy)");
    let padding = 5.; // need some padding for the outline/falloff
    let spacing = 70.;
    let palette = palettes.get(assets.palette.clone()).unwrap();

    let clear_color = palette.lightest();
    commands.insert_resource(ClearColor(clear_color));
    let mut rng = rand::thread_rng();

    for i in 0..100 {
        let size = Vec2::new(random::<f32>() * 20. + 1., random::<f32>() * 20. + 1.);
        let x = ((i % 10) as f32 - 4.5) * spacing;
        let y = ((i / 10) as f32 - 4.5) * spacing;

        let transform = Transform {
            scale: Vec3::splat(1.),
            translation: Vec3::new(x, y, 0.),
            rotation: Quat::from_rotation_z(random::<f32>() * PI),
        };

        let color = palette
            .iter()
            .filter(|c| *c != &clear_color)
            .choose(&mut rng)
            .copied()
            .unwrap_or(Color::PINK);

        commands.spawn_bundle(ShapeBundle {
            transform,
            shape: SmudShape {
                color,
                sdf: box_sdf.clone(),
                frame: Frame::Quad(f32::max(size.x, size.y) + padding),
                params: Vec4::new(size.x, size.y, 0., 0.),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
