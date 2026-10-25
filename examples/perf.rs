//! Measures frame time with many shapes, then exits.
//!
//! `cargo run --release --example perf -- <count> <mode> <combos>`
//!
//! - `count`: number of shapes, laid out in a grid that fills the window
//! - `mode`: `static` (nothing changes), `moving` (every transform changes every
//!   frame) or `recolor` (every shape's color changes every frame)
//! - `combos`: number of distinct sdf shaders, so pipelines and batches
//!
//! It renders headless into an image, so no compositor caps the frame rate, and
//! waits for the GPU at the end of every frame, so a frame time covers the CPU
//! and the GPU work. It warms up for 5 seconds (shader compilation), records
//! frame times for 10 seconds, and prints one line of stats.

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    camera::RenderTarget,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_resource::{PollType, TextureFormat},
        renderer::RenderDevice,
    },
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_smud::prelude::*;

const WARMUP_SECS: f32 = 5.;
const MEASURE_SECS: f32 = 10.;
const WIDTH: f32 = 1280.;
const HEIGHT: f32 = 720.;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Static,
    Moving,
    Recolor,
}

#[derive(Resource)]
struct Settings {
    count: usize,
    mode: Mode,
    combos: usize,
}

#[derive(Resource, Default)]
struct FrameTimes(Vec<f32>);

#[derive(Component)]
struct Index(usize);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = args.get(1).map_or(100_000, |s| s.parse().unwrap());
    let mode = match args.get(2).map_or("static", String::as_str) {
        "static" => Mode::Static,
        "moving" => Mode::Moving,
        "recolor" => Mode::Recolor,
        other => panic!("unknown mode {other}"),
    };
    let combos = args.get(3).map_or(1, |s| s.parse().unwrap());

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Settings {
            count,
            mode,
            combos,
        })
        .init_resource::<FrameTimes>()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::ZERO),
            SmudPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_shapes, recolor_shapes, measure));
    app.sub_app_mut(RenderApp)
        .add_systems(Render, wait_for_gpu.in_set(RenderSystems::Cleanup));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<Settings>,
) {
    let target = Image::new_target_texture(
        WIDTH as u32,
        HEIGHT as u32,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    commands.spawn((
        Camera2d,
        Msaa::Off,
        RenderTarget::Image(images.add(target).into()),
    ));

    // Distinct shaders that cost the same: circles of slightly different radii.
    let sdfs: Vec<_> = (0..settings.combos)
        .map(|i| {
            shaders.add_sdf_expr(format!(
                "smud::sd_circle(input.pos, input.bounds.x * {:.4})",
                0.9 - i as f32 * 0.001
            ))
        })
        .collect();

    let columns = ((settings.count as f32 * WIDTH / HEIGHT).sqrt().ceil() as usize).max(1);
    let rows = settings.count.div_ceil(columns);
    let cell = (WIDTH / columns as f32).min(HEIGHT / rows as f32);

    for index in 0..settings.count {
        let (column, row) = (index % columns, index / columns);
        commands.spawn((
            Transform::from_xyz(
                (column as f32 + 0.5) * cell - WIDTH / 2.,
                (row as f32 + 0.5) * cell - HEIGHT / 2.,
                0.,
            ),
            SmudShape {
                color: color(index, 0.),
                sdf: sdfs[index % sdfs.len()].clone(),
                fill: SIMPLE_FILL_HANDLE,
                bounds: Rectangle::from_length(cell),
                ..default()
            },
            Index(index),
        ));
    }
}

fn wait_for_gpu(render_device: Res<RenderDevice>) {
    render_device
        .poll(PollType::wait_indefinitely())
        .expect("gpu poll failed");
}

fn color(index: usize, t: f32) -> Color {
    Color::hsl((index as f32 * 7.3 + t * 90.) % 360., 0.7, 0.6)
}

fn move_shapes(
    settings: Res<Settings>,
    time: Res<Time>,
    mut shapes: Query<&mut Transform, With<SmudShape>>,
) {
    if settings.mode != Mode::Moving {
        return;
    }
    let angle = time.elapsed_secs();
    for mut transform in &mut shapes {
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn recolor_shapes(
    settings: Res<Settings>,
    time: Res<Time>,
    mut shapes: Query<(&mut SmudShape, &Index)>,
) {
    if settings.mode != Mode::Recolor {
        return;
    }
    let t = time.elapsed_secs();
    for (mut shape, index) in &mut shapes {
        shape.color = color(index.0, t);
    }
}

fn measure(
    time: Res<Time<Real>>,
    settings: Res<Settings>,
    mut frame_times: ResMut<FrameTimes>,
    mut exit: MessageWriter<AppExit>,
) {
    let elapsed = time.elapsed_secs();
    if elapsed < WARMUP_SECS {
        return;
    }
    if elapsed < WARMUP_SECS + MEASURE_SECS {
        frame_times.0.push(time.delta_secs() * 1000.);
        return;
    }
    let mut times = frame_times.0.clone();
    times.sort_by(f32::total_cmp);
    let mean = times.iter().sum::<f32>() / times.len() as f32;
    let at = |q: f32| times[((times.len() - 1) as f32 * q) as usize];
    println!(
        "perf count={} mode={:?} combos={} frames={} mean_ms={mean:.3} median_ms={:.3} p95_ms={:.3}",
        settings.count,
        settings.mode,
        settings.combos,
        times.len(),
        at(0.5),
        at(0.95),
    );
    exit.write(AppExit::Success);
}
