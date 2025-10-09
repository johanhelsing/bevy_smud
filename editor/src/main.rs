use std::{
    collections::{BTreeMap, BTreeSet},
    f32::consts::TAU,
};

use bevy::{picking::hover::PickingInteraction, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Widget},
};
use bevy_smud::prelude::*;

const SIDE_PANEL_WIDTH: f32 = 550.0;

#[derive(Default, Resource)]
struct EditorState {
    next_shader_id: u32,
    next_shape_id: u32,
    selected_shape: u32,
    scroll_to: Option<u32>,
}

impl EditorState {
    fn next_shader(&mut self) -> u32 {
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        id
    }

    fn next_shape(&mut self) -> u32 {
        let id = self.next_shape_id;
        self.next_shape_id += 1;
        self.selected_shape = id;
        self.scroll_to = Some(id);
        id
    }
}

#[derive(Clone, Component)]
struct ShapeParams {
    id: u32,
    translation: [f32; 3],
    rotation: f32,
    color: egui::Color32,
    sdf_code: String,
    fill_code: String,
    bounds_length: f32,
    params: [f32; 4],
    blend_mode: BlendMode,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Smud Editor".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SmudPlugin)
        .add_plugins(SmudPickingPlugin)
        .add_plugins(EguiPlugin::default())
        .insert_resource(EditorState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, pick)
        .add_systems(EguiPrimaryContextPass, editor)
        .run();
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<EditorState>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Transform::from_translation(Vec3::new(-SIDE_PANEL_WIDTH / 2.0, 0.0, 0.0)),
    ));

    add_shape(&mut commands, &mut state, &mut shaders);
}

fn pick(
    mut state: ResMut<EditorState>,
    query: Query<(&ShapeParams, &PickingInteraction), Changed<PickingInteraction>>,
) {
    for (params, &interaction) in query {
        if interaction == PickingInteraction::Pressed {
            state.selected_shape = params.id;
        }
    }
}

fn editor(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut shaders: ResMut<Assets<Shader>>,
    mut query: Query<(Entity, &mut Transform, &mut SmudShape, &mut ShapeParams)>,
) -> Result {
    let mut query_by_id: BTreeMap<_, _> = query
        .iter_mut()
        .map(|(entity, transform, shape, params)| (params.id, (entity, transform, shape, params)))
        .collect();

    let ids: BTreeSet<_> = query_by_id.keys().copied().collect();

    let mut selected_params = query_by_id.remove(&state.selected_shape);

    let mut update_shader = false;

    // Build UI
    egui::SidePanel::left("side_panel")
        .default_width(SIDE_PANEL_WIDTH)
        .show(contexts.ctx_mut()?, |ui| {
            // Shortcut for compiling the shaders
            let ctrl_return = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Enter);
            if ui.input_mut(|i| i.consume_shortcut(&ctrl_return)) {
                update_shader = true;
            }

            // UI for adding/deleting/selecting shapes
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    add_shape(&mut commands, &mut state, &mut shaders);
                }

                if let Some((entity, transform, shape, params)) = &selected_params {
                    if ui.button("Copy").clicked() {
                        clone_shape(&mut commands, &mut state, transform, shape, params);
                    }

                    if ui.button("Delete").clicked() {
                        let neighbor_id = ids
                            .range(0..params.id)
                            .next_back()
                            .copied()
                            .or_else(|| ids.range(params.id + 1..).next().copied())
                            .unwrap_or_default();
                        state.selected_shape = neighbor_id;
                        state.scroll_to = Some(neighbor_id);
                        commands.entity(*entity).despawn();
                    }

                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        for &id in &ids {
                            let selector = ui.selectable_value(
                                &mut state.selected_shape,
                                id,
                                format!("Shape#{id}"),
                            );
                            if state.scroll_to == Some(id) {
                                state.scroll_to = None;
                                selector.scroll_to_me(None);
                            }
                        }
                    });
                }
            });

            ui.separator();

            // UI for changing the selected shape
            if let Some((_, _, _, params)) = &mut selected_params {
                egui::Grid::new("grid_params")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Translation:");
                        ui.horizontal(|ui| {
                            egui::DragValue::new(&mut params.translation[0])
                                .speed(5.0)
                                .ui(ui);
                            egui::DragValue::new(&mut params.translation[1])
                                .speed(5.0)
                                .ui(ui);
                            egui::DragValue::new(&mut params.translation[2])
                                .speed(1.0)
                                .ui(ui);
                        });
                        ui.end_row();

                        ui.label("Rotation:");
                        ui.add(
                            egui::DragValue::new(&mut params.rotation)
                                .min_decimals(2)
                                .speed(TAU / 50.0),
                        );
                        ui.end_row();

                        ui.label("Color:");
                        ui.color_edit_button_srgba(&mut params.color);
                        ui.end_row();

                        ui.label("Bounds length");
                        egui::Slider::new(&mut params.bounds_length, 0.0..=2000.0).ui(ui);
                        ui.end_row();

                        ui.label("Params:");
                        ui.horizontal(|ui| {
                            egui::DragValue::new(&mut params.params[0])
                                .speed(1.0)
                                .ui(ui);
                            egui::DragValue::new(&mut params.params[1])
                                .speed(1.0)
                                .ui(ui);
                            egui::DragValue::new(&mut params.params[2])
                                .speed(1.0)
                                .ui(ui);
                            egui::DragValue::new(&mut params.params[3])
                                .speed(1.0)
                                .ui(ui);
                        });
                        ui.end_row();

                        ui.label("Blend mode:");
                        egui::ComboBox::from_id_salt("blend_mode")
                            .selected_text(format!("{:?}", params.blend_mode))
                            .show_ui(ui, |ui| {
                                for blend_mode in [BlendMode::Alpha, BlendMode::Additive] {
                                    ui.selectable_value(
                                        &mut params.blend_mode,
                                        blend_mode,
                                        format!("{blend_mode:?}"),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Shader compilation:");
                        ui.horizontal(|ui| {
                            if ui.button("Compile now").clicked() {
                                update_shader = true;
                            }
                            ui.label("Or press CTRL+ENTER")
                        });
                    });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::TextEdit::multiline(&mut params.sdf_code)
                        .id(egui::Id::new("sdf_editor"))
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .ui(ui);

                    egui::TextEdit::multiline(&mut params.fill_code)
                        .id(egui::Id::new("fill_editor"))
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            }
        });

    // Apply changes
    if let Some((_, mut transform, mut shape, params)) = selected_params {
        update_shape(
            &mut state,
            &mut shaders,
            &mut transform,
            &mut shape,
            &params,
            update_shader,
        );
    }

    Ok(())
}

fn add_shape(commands: &mut Commands, state: &mut EditorState, shaders: &mut Assets<Shader>) {
    let mut transform = Transform::default();
    let mut shape = SmudShape::default();

    let params = ShapeParams {
        id: state.next_shape(),
        translation: [0.0; 3],
        rotation: 0.0,
        bounds_length: 200.0,
        color: egui::Color32::from_rgb(200, 100, 100),
        sdf_code: r#"#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    let p = input.pos;
    let params = input.params;
    
    let p_2 = vec2<f32>(abs(p.x), p.y);
    return smud::sd_circle(p_2 - vec2(20., 0.), 40.);
}"#
        .to_string(),
        fill_code: r#"#import smud

fn fill(input: smud::FillInput) -> vec4<f32> {
    let p = input.pos;
    let params = input.params;
    let distance = input.distance;
    let color = input.color;

    let d2 = 1. - (distance * 0.13);
    let alpha = clamp(d2 * d2 * d2, 0., 1.) * color.a;
    let shadow_color = 0.2 * color.rgb;
    let aaf = 0.7 / fwidth(distance);
    let c = mix(color.rgb, shadow_color, clamp(distance * aaf, 0., 1.));
    return vec4(c, alpha);
}"#
        .to_string(),
        params: [0.0; 4],
        blend_mode: BlendMode::default(),
    };

    update_shape(state, shaders, &mut transform, &mut shape, &params, true);

    commands.spawn((transform, shape, params));
}

fn clone_shape(
    commands: &mut Commands,
    state: &mut EditorState,
    transform: &Transform,
    shape: &SmudShape,
    params: &ShapeParams,
) {
    let mut params = params.clone();
    params.id = state.next_shape();

    commands.spawn((*transform, shape.clone(), params));
}

fn update_shape(
    state: &mut EditorState,
    shaders: &mut Assets<Shader>,
    transform: &mut Transform,
    shape: &mut SmudShape,
    params: &ShapeParams,
    update_shader: bool,
) {
    *transform = Transform::from_translation(Vec3::from_array(params.translation))
        .with_rotation(Quat::from_rotation_z(params.rotation));

    let [r, g, b, a] = params.color.to_array();
    shape.color = Color::srgba_u8(r, g, b, a);
    shape.bounds = Rectangle::from_length(params.bounds_length);
    shape.params = Vec4::from_array(params.params);
    shape.blend_mode = params.blend_mode;

    if update_shader {
        let mut sdf_shader_code =
            format!("#define_import_path smud::sdf_{}\n", state.next_shader());
        sdf_shader_code.push_str(&params.sdf_code);
        let sdf_shader = Shader::from_wgsl(sdf_shader_code, file!());
        shape.sdf = shaders.add(sdf_shader);

        let mut fill_shader_code =
            format!("#define_import_path smud::fill_{}\n", state.next_shader());
        fill_shader_code.push_str(&params.fill_code);
        let fill_shader = Shader::from_wgsl(fill_shader_code, file!());
        shape.fill = shaders.add(fill_shader);
    }
}
