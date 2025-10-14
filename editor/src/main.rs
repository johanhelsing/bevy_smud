use std::{collections::BTreeSet, f32::consts::TAU, fmt::Display};

use bevy::{picking::hover::PickingInteraction, prelude::*};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, Widget},
};
use bevy_smud::prelude::*;
use include_dir::include_dir;

const SIDE_PANEL_WIDTH: f32 = 550.0;
const DEFAULT_SDF_TEMPLATE: &str = "circle";
const DEFAULT_FILL_TEMPLATE: &str = "simple";
static SDF_TEMPLATE_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/sdf");
static FILL_TEMPLATE_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/fill");

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
            background_color: egui::Color32::from_rgb(43, 44, 47), // Same as default ClearColor
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
struct ShapeState {
    id: u32,
    position: Vec3,
    rotation: f32,
    scale: f32,
    color: egui::Color32,
    selected_shader: ShaderKind,
    sdf_code: String,
    fill_code: String,
    bounds_length: f32,
    params: Vec4,
    blend_mode: BlendMode,
}

#[derive(Clone, Copy, PartialEq)]
enum ShaderKind {
    Sdf,
    Fill,
}

impl Display for ShaderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderKind::Sdf => write!(f, "sdf"),
            ShaderKind::Fill => write!(f, "fill"),
        }
    }
}

#[derive(Component)]
struct ShapeCamera;

#[derive(Resource)]
struct Templates {
    sdf: Vec<Template>,
    fill: Vec<Template>,
    default_sdf: Option<usize>,
    default_fill: Option<usize>,
}

impl Default for Templates {
    fn default() -> Self {
        let sdf: Vec<_> = SDF_TEMPLATE_DIR.files().map(Template::new).collect();
        let fill: Vec<_> = FILL_TEMPLATE_DIR.files().map(Template::new).collect();
        let default_sdf = sdf.iter().position(|t| t.name == DEFAULT_SDF_TEMPLATE);
        let default_fill = fill.iter().position(|t| t.name == DEFAULT_FILL_TEMPLATE);
        Self {
            sdf,
            fill,
            default_sdf,
            default_fill,
        }
    }
}

impl Templates {
    fn all_templates(&self, shader: ShaderKind) -> &[Template] {
        match shader {
            ShaderKind::Sdf => &self.sdf,
            ShaderKind::Fill => &self.fill,
        }
    }

    fn default_template(&self, shader: ShaderKind) -> Option<&Template> {
        let (all, index) = match shader {
            ShaderKind::Sdf => (&self.sdf, self.default_sdf),
            ShaderKind::Fill => (&self.fill, self.default_fill),
        };
        index.and_then(|i| all.get(i))
    }
}

struct Template {
    name: String,
    code: String,
}

impl Template {
    fn new(file: &include_dir::File) -> Self {
        let name = file
            .path()
            .file_stem()
            .expect("Template must be a .wgsl file")
            .to_string_lossy()
            .into_owned();
        let code = file
            .contents_utf8()
            .expect("Template must contain valid utf-8")
            .to_owned();
        Self { name, code }
    }
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
        .insert_resource(Templates::default())
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
    templates: Res<Templates>,
    mut editor_state: ResMut<EditorState>,
    mut clear_color: ResMut<ClearColor>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    clear_color.0 = convert_color(editor_state.background_color);

    commands.spawn((
        ShapeCamera,
        Camera2d,
        Msaa::Off,
        Transform::from_translation(editor_state.camera_position.extend(0.0)),
    ));

    add_shape(&mut commands, &templates, &mut editor_state, &mut shaders);
}

fn pick(
    mut editor_state: ResMut<EditorState>,
    query: Query<(&ShapeState, &PickingInteraction), Changed<PickingInteraction>>,
) {
    for (shape_state, &interaction) in query {
        if interaction == PickingInteraction::Pressed {
            editor_state.selected_tab = SelectedTab::Shape(shape_state.id);
        }
    }
}

fn camera(
    editor_state: Res<EditorState>,
    mut camera_query: Single<&mut Transform, With<ShapeCamera>>,
) {
    let camera_transform = camera_query.as_mut();
    *camera_transform = Transform::from_translation(editor_state.camera_position.extend(0.0));
}

fn background(editor_state: Res<EditorState>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = convert_color(editor_state.background_color);
}

fn editor(
    mut commands: Commands,
    mut contexts: EguiContexts,
    templates: Res<Templates>,
    mut editor_state: ResMut<EditorState>,
    mut shaders: ResMut<Assets<Shader>>,
    mut shape_query: Query<(Entity, &mut Transform, &mut SmudShape, &mut ShapeState)>,
) -> Result {
    let padding = 4.0;

    // Build UI
    egui::SidePanel::left("side_panel")
        .default_width(SIDE_PANEL_WIDTH)
        .show(contexts.ctx_mut()?, |ui| {
            // UI for selecting/editing tabs
            ui.add_space(padding);

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut editor_state.selected_tab,
                    SelectedTab::Global,
                    "Global",
                );

                ui.separator();

                if ui.button("Add").clicked() {
                    add_shape(&mut commands, &templates, &mut editor_state, &mut shaders);
                }

                let shapes: BTreeSet<_> = shape_query
                    .iter()
                    .map(|(_, _, _, shape_state)| shape_state.id)
                    .collect();

                let selected_shape = match editor_state.selected_tab {
                    SelectedTab::Shape(id) => Some(id),
                    _ => None,
                };

                ui.add_enabled_ui(selected_shape.is_some(), |ui| {
                    if ui.button("Copy").clicked()
                        && let Some(id) = selected_shape
                        && let Some((transform, shape, shape_state)) =
                            shape_query
                                .iter()
                                .find_map(|(_, transform, shape, shape_state)| {
                                    (shape_state.id == id).then_some((
                                        transform,
                                        shape,
                                        shape_state,
                                    ))
                                })
                    {
                        clone_shape(
                            &mut commands,
                            &mut editor_state,
                            transform,
                            shape,
                            shape_state,
                        );
                    }

                    if ui.button("Delete").clicked()
                        && let Some(id) = selected_shape
                        && let Some(entity) =
                            shape_query.iter().find_map(|(entity, _, _, shape_state)| {
                                (shape_state.id == id).then_some(entity)
                            })
                    {
                        let neighbor_id = shapes
                            .range(0..id)
                            .next_back()
                            .copied()
                            .or_else(|| shapes.range(id + 1..).next().copied());
                        editor_state.select_tab(
                            neighbor_id.map_or(SelectedTab::Global, SelectedTab::Shape),
                        );
                        commands.entity(entity).despawn();
                    }
                });

                egui::ScrollArea::horizontal()
                    .id_salt("scroll_tab")
                    .show(ui, |ui| {
                        for id in shapes {
                            let selector = ui.selectable_value(
                                &mut editor_state.selected_tab,
                                SelectedTab::Shape(id),
                                format!("shape_{id}"),
                            );
                            if editor_state.scroll_to == Some(id) {
                                editor_state.scroll_to = None;
                                selector.scroll_to_me(None);
                            }
                        }
                    });
            });

            ui.separator();

            match editor_state.selected_tab {
                SelectedTab::Global => {
                    // UI for changing global settings
                    egui::Grid::new("grid_global")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Camera position:");
                            ui.horizontal(|ui| {
                                egui::DragValue::new(&mut editor_state.camera_position.x)
                                    .speed(5.0)
                                    .ui(ui);
                                egui::DragValue::new(&mut editor_state.camera_position.y)
                                    .speed(5.0)
                                    .ui(ui);
                            });
                            ui.end_row();

                            ui.label("Background color:");
                            ui.color_edit_button_srgba(&mut editor_state.background_color);
                            ui.end_row();
                        });
                }
                SelectedTab::Shape(id) => {
                    // UI for changing the selected shape
                    if let Some((_, mut transform, mut shape, mut shape_state)) = shape_query
                        .iter_mut()
                        .find(|(_, _, _, shape_state)| shape_state.id == id)
                    {
                        egui::Grid::new("grid_shape")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Position:");
                                ui.horizontal(|ui| {
                                    egui::DragValue::new(&mut shape_state.position.x)
                                        .speed(5.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut shape_state.position.y)
                                        .speed(5.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut shape_state.position.z)
                                        .speed(1.0)
                                        .ui(ui);
                                });
                                ui.end_row();

                                ui.label("Rotation:");
                                ui.add(
                                    egui::DragValue::new(&mut shape_state.rotation)
                                        .min_decimals(2)
                                        .speed(TAU / 50.0),
                                );
                                ui.end_row();

                                ui.label("Scale:");
                                ui.add(
                                    egui::DragValue::new(&mut shape_state.scale)
                                        .min_decimals(1)
                                        .speed(1.0 / 5.0),
                                );
                                ui.end_row();

                                ui.label("Color:");
                                ui.color_edit_button_srgba(&mut shape_state.color);
                                ui.end_row();

                                ui.label("Bounds length:");
                                egui::Slider::new(&mut shape_state.bounds_length, 0.0..=2000.0)
                                    .ui(ui);
                                ui.end_row();

                                ui.label("Params:");
                                ui.horizontal(|ui| {
                                    egui::DragValue::new(&mut shape_state.params[0])
                                        .speed(1.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut shape_state.params[1])
                                        .speed(1.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut shape_state.params[2])
                                        .speed(1.0)
                                        .ui(ui);
                                    egui::DragValue::new(&mut shape_state.params[3])
                                        .speed(1.0)
                                        .ui(ui);
                                });
                                ui.end_row();

                                ui.label("Blend mode:");
                                egui::ComboBox::from_id_salt("blend_mode")
                                    .selected_text(format!("{:?}", shape_state.blend_mode))
                                    .show_ui(ui, |ui| {
                                        for blend_mode in [BlendMode::Alpha, BlendMode::Additive] {
                                            ui.selectable_value(
                                                &mut shape_state.blend_mode,
                                                blend_mode,
                                                format!("{blend_mode:?}"),
                                            );
                                        }
                                    });
                                ui.end_row();
                            });

                        ui.separator();

                        let mut compile_shader = false;

                        ui.horizontal(|ui| {
                            for shader in [ShaderKind::Sdf, ShaderKind::Fill] {
                                ui.selectable_value(
                                    &mut shape_state.selected_shader,
                                    shader,
                                    format!("{shader}"),
                                );
                            }

                            ui.separator();

                            if ui.button("Compile").clicked() {
                                compile_shader = true;
                            }

                            ui.label("or press ctrl+enter");
                            let ctrl_return = egui::KeyboardShortcut::new(
                                egui::Modifiers::CTRL,
                                egui::Key::Enter,
                            );
                            if ui.input_mut(|i| i.consume_shortcut(&ctrl_return)) {
                                compile_shader = true;
                            }

                            ui.separator();

                            let template_button = ui.button("Template");
                            egui::Popup::menu(&template_button)
                                .id(egui::Id::new(format!(
                                    "template_menu_{}",
                                    shape_state.selected_shader
                                ))) // Each popup should have its own state
                                .align(egui::RectAlign::BOTTOM_START)
                                .gap(padding)
                                .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                                .show(|ui| {
                                    egui::ScrollArea::vertical()
                                        .max_height(300.0)
                                        .show(ui, |ui| {
                                            for template in
                                                templates.all_templates(shape_state.selected_shader)
                                            {
                                                if ui.button(&template.name).clicked() {
                                                    let code = match shape_state.selected_shader {
                                                        ShaderKind::Sdf => {
                                                            &mut shape_state.sdf_code
                                                        }
                                                        ShaderKind::Fill => {
                                                            &mut shape_state.fill_code
                                                        }
                                                    };
                                                    code.clear();
                                                    code.push_str(&template.code);
                                                }
                                            }
                                        })
                                });
                        });

                        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
                            ui.ctx(),
                            ui.style(),
                        );

                        let mut layouter =
                            |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                                let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                    ui.ctx(),
                                    ui.style(),
                                    &theme,
                                    buf.as_str(),
                                    "rs", // There is no highlighter for wgsl yet
                                );
                                layout_job.wrap.max_width = wrap_width;
                                ui.fonts_mut(|f| f.layout_job(layout_job))
                            };

                        let code = match shape_state.selected_shader {
                            ShaderKind::Sdf => &mut shape_state.sdf_code,
                            ShaderKind::Fill => &mut shape_state.fill_code,
                        };
                        egui::Frame::new()
                            .inner_margin(egui::vec2(0.0, padding))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().id_salt("scroll_editor").show(
                                    ui,
                                    |ui| {
                                        ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(code)
                                                .id(egui::Id::new("editor"))
                                                .font(egui::TextStyle::Monospace) // for cursor height
                                                .code_editor()
                                                .lock_focus(true)
                                                .layouter(&mut layouter),
                                        );
                                    },
                                );
                            });

                        // Apply changes
                        update_shape(
                            &mut editor_state,
                            &mut shaders,
                            &mut transform,
                            &mut shape,
                            &shape_state,
                            compile_shader,
                        );
                    }
                }
            };
        });

    Ok(())
}

fn add_shape(
    commands: &mut Commands,
    templates: &Templates,
    state: &mut EditorState,
    shaders: &mut Assets<Shader>,
) {
    let mut transform = Transform::default();
    let mut shape = SmudShape::default();

    let shape_state = ShapeState {
        id: state.create_shape(),
        position: Vec3::ZERO,
        rotation: 0.0,
        scale: 1.0,
        bounds_length: 500.0,
        color: egui::Color32::from_rgb(200, 100, 100),
        selected_shader: ShaderKind::Sdf,
        sdf_code: templates
            .default_template(ShaderKind::Sdf)
            .map(|t| t.code.clone())
            .unwrap_or_default(),
        fill_code: templates
            .default_template(ShaderKind::Fill)
            .map(|t| t.code.clone())
            .unwrap_or_default(),
        params: Vec4::ZERO,
        blend_mode: BlendMode::default(),
    };

    update_shape(
        state,
        shaders,
        &mut transform,
        &mut shape,
        &shape_state,
        true,
    );

    commands.spawn((transform, shape, shape_state));
}

fn clone_shape(
    commands: &mut Commands,
    state: &mut EditorState,
    transform: &Transform,
    shape: &SmudShape,
    shape_state: &ShapeState,
) {
    let mut shape_state = shape_state.clone();
    shape_state.id = state.create_shape();

    commands.spawn((*transform, shape.clone(), shape_state));
}

fn update_shape(
    editor_state: &mut EditorState,
    shaders: &mut Assets<Shader>,
    transform: &mut Transform,
    shape: &mut SmudShape,
    shape_state: &ShapeState,
    compile_shader: bool,
) {
    *transform = Transform::from_translation(shape_state.position)
        .with_rotation(Quat::from_rotation_z(shape_state.rotation))
        .with_scale(Vec3::splat(shape_state.scale));

    shape.color = convert_color(shape_state.color);
    shape.bounds = Rectangle::from_length(shape_state.bounds_length);
    shape.params = shape_state.params;
    shape.blend_mode = shape_state.blend_mode;

    if compile_shader {
        let sdf_shader_code = add_unique_shader_import_path(&shape_state.sdf_code, editor_state);
        let sdf_shader = Shader::from_wgsl(sdf_shader_code, file!());
        shape.sdf = shaders.add(sdf_shader);

        let fill_shader_code = add_unique_shader_import_path(&shape_state.fill_code, editor_state);
        let fill_shader = Shader::from_wgsl(fill_shader_code, file!());
        shape.fill = shaders.add(fill_shader);
    }
}

fn convert_color(color: egui::Color32) -> Color {
    let [r, g, b, a] = color.to_array();
    Color::srgba_u8(r, g, b, a)
}

fn add_unique_shader_import_path(code: &str, editor_state: &mut EditorState) -> String {
    let id = editor_state.create_shader();
    let import_path_directive = "#define_import_path ";
    let unique_shader_import_path = format!("{import_path_directive}smud_editor::shader_{id}\n");
    let mut result = unique_shader_import_path;
    for line in code.lines() {
        if !line.contains("#define_import_path") {
            result.push_str(line);
            result.push_str("\n");
        }
    }
    result
}
