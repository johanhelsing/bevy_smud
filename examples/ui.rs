use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SmudPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let circle_sdf = asset_server.load("shapes/circle.wgsl");
    let ellipse_sdf = asset_server.load("shapes/ellipse.wgsl");

    // Container for all elements
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(50.0),
                    top: Val::Px(50.0),
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    ..default()
                },
                SmudNode {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    sdf: circle_sdf.clone(),
                    params: Vec4::new(100.0, 0.0, 0.0, 0.0),
                    ..default()
                },
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(100.0),
                    top: Val::Px(100.0),
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    ..default()
                },
                SmudNode {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    sdf: ellipse_sdf.clone(),
                    params: Vec4::new(100.0, 50.0, 0.0, 0.0),
                    ..default()
                },
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(150.0),
                    top: Val::Px(75.0),
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    ..default()
                },
                SmudNode {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    sdf: circle_sdf,
                    params: Vec4::new(100.0, 0.0, 0.0, 0.0),
                    ..default()
                },
            )
        ],
    ));
}
