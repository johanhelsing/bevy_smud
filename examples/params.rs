use std::f32::consts::PI;

use bevy::color::palettes::css as Colors;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_smud::prelude::*;
use rand::{prelude::IteratorRandom, random};

// this example shows how to use per-instance parameters in shapes
// in this simple example, a width and height is passed to a box shape,
// but it could be used for almost anything.

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin, bevy_lospec::PalettePlugin))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Running)
                .load_collection::<AssetHandles>(),
        )
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

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    assets: Res<AssetHandles>,
    palettes: Res<Assets<bevy_lospec::Palette>>,
) {
    let box_sdf = shaders.add_sdf_expr("smud::sd_box(p, params.xy)");
    let padding = 5.; // need some padding for the outline/falloff
    let spacing = 70.;
    let palette = palettes.get(&assets.palette).unwrap();

    let clear_color = palette.lightest();
    commands.insert_resource(ClearColor(clear_color));
    let mut rng = rand::rng();

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
            .unwrap_or(Colors::PINK.into());

        commands.spawn((
            transform,
            SmudShape {
                color,
                sdf: box_sdf.clone(),
                frame: Frame::Quad(f32::max(size.x, size.y) + padding),
                params: Vec4::new(size.x, size.y, 0., 0.),
                ..default()
            },
        ));
    }

    commands.spawn(Camera2d);
}
