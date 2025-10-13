use std::{collections::BTreeSet, f32::consts::TAU};

use bevy::{picking::hover::PickingInteraction, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Color32, Widget},
};
use bevy_smud::prelude::*;

const SIDE_PANEL_WIDTH: f32 = 550.0;

type ShaderId = u32;
type ShapeId = u32;

#[derive(Resource)]
struct EditorState {
    camera_position: Vec2,
    background_color: egui::Color32,
    next_shader_id: ShapeId,
    next_shape_id: ShapeId,
    selected_tab: SelectedTab,
    scroll_to: Option<ShapeId>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            camera_position: Vec2::new(-SIDE_PANEL_WIDTH / 2.0, 0.0),
            background_color: Color32::from_rgb(43, 44, 47), // Same as default ClearColor
            next_shader_id: 0,
            next_shape_id: 0,
            selected_tab: SelectedTab::Global,
            scroll_to: None,
        }
    }
}

impl EditorState {
    fn create_shader(&mut self) -> ShaderId {
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        id
    }

    fn create_shape(&mut self) -> ShapeId {
        let id = self.next_shape_id;
        self.next_shape_id += 1;
        self.select_tab(SelectedTab::Shape(id));
        id
    }

    fn select_tab(&mut self, tab: SelectedTab) {
        self.selected_tab = tab;
        if let SelectedTab::Shape(id) = tab {
            self.scroll_to = Some(id);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SelectedTab {
    Global,
    Shape(u32),
}

#[derive(Clone, Component)]
struct ShapeParams {
    id: u32,
    position: Vec3,
    rotation: f32,
    scale: f32,
    color: egui::Color32,
    sdf_code: String,
    fill_code: String,
    bounds_length: f32,
    params: Vec4,
    blend_mode: BlendMode,
}

#[derive(Component)]
struct ShapeCamera;

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
        .add_systems(Update, camera)
        .add_systems(Update, background)
        .add_systems(EguiPrimaryContextPass, editor)
        .run();
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<EditorState>,
    mut clear_color: ResMut<ClearColor>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    clear_color.0 = convert_color(state.background_color);

    commands.spawn((
        ShapeCamera,
        Camera2d,
        Msaa::Off,
        Transform::from_translation(state.camera_position.extend(0.0)),
    ));

    add_shape(&mut commands, &mut state, &mut shaders);
}

fn pick(
    mut state: ResMut<EditorState>,
    query: Query<(&ShapeParams, &PickingInteraction), Changed<PickingInteraction>>,
) {
    for (params, &interaction) in query {
        if interaction == PickingInteraction::Pressed {
            state.selected_tab = SelectedTab::Shape(params.id);
        }
    }
}

fn camera(state: Res<EditorState>, mut camera_query: Single<&mut Transform, With<ShapeCamera>>) {
    let camera_transform = camera_query.as_mut();
    *camera_transform = Transform::from_translation(state.camera_position.extend(0.0));
}

fn background(state: Res<EditorState>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = convert_color(state.background_color);
}

fn editor(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut shaders: ResMut<Assets<Shader>>,
    mut shape_query: Query<(Entity, &mut Transform, &mut SmudShape, &mut ShapeParams)>,
) -> Result {
    // Build UI
    egui::SidePanel::left("side_panel")
        .default_width(SIDE_PANEL_WIDTH)
        .show(contexts.ctx_mut()?, |ui| {
            // UI for selecting/editing tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.selected_tab, SelectedTab::Global, "Global");

                ui.separator();

                if ui.button("Add").clicked() {
                    add_shape(&mut commands, &mut state, &mut shaders);
                }

                let shapes: BTreeSet<_> = shape_query
                    .iter()
                    .map(|(_, _, _, params)| params.id)
                    .collect();

                let selected_shape = match state.selected_tab {
                    SelectedTab::Shape(id) => Some(id),
                    _ => None,
                };

                ui.add_enabled_ui(selected_shape.is_some(), |ui| {
                    if ui.button("Copy").clicked()
                        && let Some(id) = selected_shape
                        && let Some((transform, shape, params)) =
                            shape_query
                                .iter()
                                .find_map(|(_, transform, shape, params)| {
                                    (params.id == id).then_some((transform, shape, params))
                                })
                    {
                        clone_shape(&mut commands, &mut state, transform, shape, params);
                    }

                    if ui.button("Delete").clicked()
                        && let Some(id) = selected_shape
                        && let Some(entity) = shape_query
                            .iter()
                            .find_map(|(entity, _, _, params)| (params.id == id).then_some(entity))
                    {
                        let neighbor_id = shapes
                            .range(0..id)
                            .next_back()
                            .copied()
                            .or_else(|| shapes.range(id + 1..).next().copied());
                        state.select_tab(
                            neighbor_id.map_or(SelectedTab::Global, SelectedTab::Shape),
                        );
                        commands.entity(entity).despawn();
                    }
                });

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for id in shapes {
                        let selector = ui.selectable_value(
                            &mut state.selected_tab,
                            SelectedTab::Shape(id),
                            format!("Shape#{id}"),
                        );
                        if state.scroll_to == Some(id) {
                            state.scroll_to = None;
                            selector.scroll_to_me(None);
                        }
                    }
                });
            });

            ui.separator();

            match state.selected_tab {
                SelectedTab::Global => {
                    // UI for changing global settings
                    egui::Grid::new("grid_params")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Camera position:");
                            ui.horizontal(|ui| {
                                egui::DragValue::new(&mut state.camera_position.x)
                                    .speed(5.0)
                                    .ui(ui);
                                egui::DragValue::new(&mut state.camera_position.y)
                                    .speed(5.0)
                                    .ui(ui);
                            });
                            ui.end_row();

                            ui.label("Background color:");
                            ui.color_edit_button_srgba(&mut state.background_color);
                            ui.end_row();
                        });
                }
                SelectedTab::Shape(id) => {
                    // UI for changing the selected shape
                    let mut update_shader = false;

                    // Shortcut for compiling the shaders
                    let ctrl_return =
                        egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Enter);
                    if ui.input_mut(|i| i.consume_shortcut(&ctrl_return)) {
                        update_shader = true;
                    }

                    if let Some((_, mut transform, mut shape, mut params)) = shape_query
                        .iter_mut()
                        .find(|(_, _, _, params)| params.id == id)
                    {
                        egui::Grid::new("grid_params")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Position:");
                                ui.horizontal(|ui| {
                                    egui::DragValue::new(&mut params.position.x)
                                        .speed(5.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut params.position.y)
                                        .speed(5.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut params.position.z)
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

                                ui.label("Scale:");
                                ui.add(
                                    egui::DragValue::new(&mut params.scale)
                                        .min_decimals(1)
                                        .speed(1.0 / 5.0),
                                );
                                ui.end_row();

                                ui.label("Color:");
                                ui.color_edit_button_srgba(&mut params.color);
                                ui.end_row();

                                ui.label("Bounds length:");
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

                        // Apply changes
                        update_shape(
                            &mut state,
                            &mut shaders,
                            &mut transform,
                            &mut shape,
                            &params,
                            update_shader,
                        );
                    }
                }
            };
        });

    Ok(())
}

fn add_shape(commands: &mut Commands, state: &mut EditorState, shaders: &mut Assets<Shader>) {
    let mut transform = Transform::default();
    let mut shape = SmudShape::default();

    let params = ShapeParams {
        id: state.create_shape(),
        position: Vec3::ZERO,
        rotation: 0.0,
        scale: 1.0,
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
        params: Vec4::ZERO,
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
    params.id = state.create_shape();

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
    *transform = Transform::from_translation(params.position)
        .with_rotation(Quat::from_rotation_z(params.rotation))
        .with_scale(Vec3::splat(params.scale));

     shape.color = convert_color(params.color);
     shape.bounds = Rectangle::from_length(params.bounds_length);
     shape.params = params.params;
     shape.blend_mode = params.blend_mode;

    if update_shader {
        let mut sdf_shader_code =
            format!("#define_import_path smud::sdf_{}\n", state.create_shader());
        sdf_shader_code.push_str(&params.sdf_code);
        let sdf_shader = Shader::from_wgsl(sdf_shader_code, file!());
        shape.sdf = shaders.add(sdf_shader);

        let mut fill_shader_code =
            format!("#define_import_path smud::fill_{}\n", state.create_shader());
        fill_shader_code.push_str(&params.fill_code);
        let fill_shader = Shader::from_wgsl(fill_shader_code, file!());
        shape.fill = shaders.add(fill_shader);
    }
}

fn convert_color(color: egui::Color32) -> Color {
    let [r, g, b, a] = color.to_array();
    Color::srgba_u8(r, g, b, a)
}
