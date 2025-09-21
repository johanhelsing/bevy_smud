use bevy::prelude::*;
use bevy_smud::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SmudPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    commands.spawn(Camera2d);

    let circle = shaders.add_sdf_expr("smud::sd_circle(input.pos, 50.)");
    let position_test_fill = shaders.add_fill_body(
        r#"
        let a = smud::sd_fill_alpha_fwidth(input.distance);

        // Use position to create a gradient effect
        let gradient = (input.pos.x + input.pos.y) * 0.01 + 0.5;

        // Use fill_params for color modulation
        let color_mod = input.params.x * 0.5 + 0.5;

        let final_color = vec3<f32>(
            input.color.r * gradient,
            input.color.g * color_mod, 
            input.color.b
        );

        return vec4<f32>(final_color, a * input.color.a);
    "#,
    );

    commands.spawn(SmudShape {
        color: Color::WHITE,
        sdf: circle,
        fill: position_test_fill,
        params: Vec4::new(1.0, 0.0, 0.0, 0.0), // Red modulation
        frame: Frame::Quad(55.),
        ..default()
    });
}
