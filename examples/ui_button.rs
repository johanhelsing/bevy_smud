use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin, SmudPickingPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, button_interaction)
        .run();
}

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    // bounds is a vec2<f32> containing the half-extents of the node,
    // making the rounded box automatically scale to the node size
    let sdf = shaders.add_sdf_expr("smud::sd_rounded_box(p, bounds, vec4<f32>(15.))");

    // fill shader with outline and animated diagonal lines
    let fill = shaders.add_fill_body(
        r#"
let outline_width = 5.;
let outline = abs(d + outline_width) - outline_width;

// diagonal scrolling lines
let h = (p.x + p.y) * 0.70710678118 + time * 20.;
let line_spacing = 20.;
let line_width = 5.;
let line_dist = abs((fract(h / line_spacing) - 0.5) * line_spacing) - line_width;;

let lines_inside_shape = max(line_dist, d);

let outline_and_lines = min(lines_inside_shape, outline);

let a = smud::sd_fill_alpha_fwidth(outline_and_lines);
return vec4<f32>(input.color.rgb, a * input.color.a);
        "#,
    );

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            // Regular button
            (
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiShape {
                    color: css::CORNFLOWER_BLUE.into(),
                    sdf,
                    fill: fill.clone(),
                    ..default()
                },
                children![(
                    Text::new("Click Me!"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(css::WHITE.into()),
                )],
            ),
            // Bevy button
            (
                Button,
                Node {
                    width: Val::Px(450.0),
                    height: Val::Px(450.0),
                    ..default()
                },
                UiShape {
                    color: css::CORNFLOWER_BLUE.into(),
                    sdf: asset_server.load("bevy.wgsl"),
                    fill,
                    ..default()
                },
            )
        ],
    ));
}

fn button_interaction(mut query: Query<(&mut UiShape, &Interaction), Changed<Interaction>>) {
    for (mut shape, interaction) in query.iter_mut() {
        shape.color = match interaction {
            Interaction::Pressed => css::DARK_BLUE.into(),
            Interaction::Hovered => css::DODGER_BLUE.into(),
            Interaction::None => css::CORNFLOWER_BLUE.into(),
        };
    }
}
