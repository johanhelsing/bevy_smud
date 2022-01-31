use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.75)))
        .add_plugins(DefaultPlugins)
        .add_plugin(SmudPlugin)
        .add_plugin(PanCamPlugin)
        .add_startup_system(setup)
        .add_system(button_system)
        .run();
}

const NORMAL_BUTTON: Color = Color::rgb(0.9, 0.9, 0.9);
const HOVERED_BUTTON: Color = Color::WHITE;
const PRESSED_BUTTON: Color = Color::rgba(1., 1., 1., 0.8);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bevy_shape_shader = asset_server.load("bevy.wgsl");

    commands
        .spawn_bundle(UiShapeBundle {
            style: Style {
                size: Size::new(Val::Px(600.0), Val::Px(450.0)),
                justify_content: JustifyContent::SpaceBetween,
                margin: Rect::all(Val::Auto),
                ..Default::default()
            },
            shape: SmudShape {
                color: Color::rgb(0.9, 0.5, 0.4),
                sdf: bevy_shape_shader,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Button)
        .insert(Interaction::default());

    commands.spawn_bundle(UiCameraBundle::default());
}
