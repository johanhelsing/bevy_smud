use bevy::{prelude::*, reflect::TypeUuid};

const PRELUDE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11291576006157771079);
const PRELUDE_SHADER_IMPORT: &str = "bevy_smud::prelude";

const SHAPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10055894596049459186);
const SHAPES_SHADER_IMPORT: &str = "bevy_smud::shapes";

const COLORIZE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10050447940405429418);
const COLORIZE_SHADER_IMPORT: &str = "bevy_smud::colorize";

pub const SMUD_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5645555317811706725);
const SMUD_SHADER_IMPORT: &str = "bevy_smud::smud";

pub const VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16846632126033267571);
const VERTEX_SHADER_IMPORT: &str = "bevy_smud::vertex";

pub const FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10370213491934870425);
const FRAGMENT_SHADER_IMPORT: &str = "bevy_smud::fragment";

pub const DEFAULT_FILL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 18184663565780163454);
const DEFAULT_FILL_IMPORT: &str = "bevy_smud::default_fill";

// unused:
// 16286090377316294491
// 16950619110804285379
// 4146091551367169642
// 8080191226000727371
// 17031499878237077924
// 17982773815777006860
// 1530570659737977289

#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShader {
    strong_handle: Handle<Shader>,
    untyped_handle: Option<HandleUntyped>,
    loaded: bool,
    import_path: String,
}

// Needed to keep the shaders alive
#[cfg(feature = "smud_shader_hot_reloading")]
struct HotShaders<T> {
    shaders: Vec<HotShader>,
    marker: std::marker::PhantomData<T>,
}

#[cfg(feature = "smud_shader_hot_reloading")]
impl<T> Default for HotShaders<T> {
    fn default() -> Self {
        Self {
            shaders: Default::default(),
            marker: Default::default(),
        }
    }
}

#[cfg(feature = "smud_shader_hot_reloading")]
fn setup_shader_imports<T: 'static + Send + Sync>(
    mut hot_shaders: ResMut<HotShaders<T>>,
    mut shaders: ResMut<Assets<Shader>>,
    asset_server: Res<AssetServer>,
) {
    for hot_shader in hot_shaders.shaders.iter_mut() {
        if !hot_shader.loaded
            && asset_server.get_load_state(hot_shader.strong_handle.clone())
                == bevy::asset::LoadState::Loaded
        {
            shaders
                .get_mut(hot_shader.strong_handle.clone())
                .unwrap()
                .set_import_path(&hot_shader.import_path);

            hot_shader.loaded = true;
        }
    }
}

pub struct ShaderLoadingPlugin;

impl Plugin for ShaderLoadingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "smud_shader_hot_reloading")]
        {
            let mut hot_shaders = {
                let asset_server = app.world.get_resource::<AssetServer>().unwrap();
                HotShaders::<Self> {
                    shaders: [
                        ("prelude.wgsl", PRELUDE_SHADER_IMPORT, PRELUDE_SHADER_HANDLE),
                        ("shapes.wgsl", SHAPES_SHADER_IMPORT, SHAPES_SHADER_HANDLE),
                        (
                            "colorize.wgsl",
                            COLORIZE_SHADER_IMPORT,
                            COLORIZE_SHADER_HANDLE,
                        ),
                        ("smud.wgsl", SMUD_SHADER_IMPORT, SMUD_SHADER_HANDLE),
                        ("vertex.wgsl", VERTEX_SHADER_IMPORT, VERTEX_SHADER_HANDLE),
                        (
                            "fragment.wgsl",
                            FRAGMENT_SHADER_IMPORT,
                            FRAGMENT_SHADER_HANDLE,
                        ),
                        (
                            "fills/cubic_falloff.wgsl",
                            DEFAULT_FILL_IMPORT,
                            DEFAULT_FILL_HANDLE,
                        ),
                    ]
                    .into_iter()
                    .map(|(path, import_path, untyped_handle)| HotShader {
                        strong_handle: asset_server.load(path),
                        untyped_handle: Some(untyped_handle),
                        import_path: import_path.into(),
                        loaded: false,
                    })
                    .collect(),
                    ..Default::default()
                }
            };
            let mut shader_assets = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            for hot_shader in hot_shaders.shaders.iter_mut() {
                let untyped_handle = hot_shader.untyped_handle.take().unwrap();
                shader_assets.add_alias(hot_shader.strong_handle.clone(), untyped_handle);
            }

            app.insert_resource(hot_shaders);
            app.add_system(setup_shader_imports::<Self>);
        }

        #[cfg(not(feature = "smud_shader_hot_reloading"))]
        {
            let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            let prelude = Shader::from_wgsl(include_str!("../assets/prelude.wgsl"))
                .with_import_path(PRELUDE_SHADER_IMPORT);
            shaders.set_untracked(PRELUDE_SHADER_HANDLE, prelude);

            let shapes = Shader::from_wgsl(include_str!("../assets/shapes.wgsl"))
                .with_import_path(SHAPES_SHADER_IMPORT);
            shaders.set_untracked(SHAPES_SHADER_HANDLE, shapes);

            let colorize = Shader::from_wgsl(include_str!("../assets/colorize.wgsl"))
                .with_import_path(COLORIZE_SHADER_IMPORT);
            shaders.set_untracked(COLORIZE_SHADER_HANDLE, colorize);

            let smud = Shader::from_wgsl(include_str!("../assets/smud.wgsl"))
                .with_import_path(SMUD_SHADER_IMPORT);
            shaders.set_untracked(SMUD_SHADER_HANDLE, smud);

            let vertex = Shader::from_wgsl(include_str!("../assets/vertex.wgsl"))
                .with_import_path(VERTEX_SHADER_IMPORT);
            shaders.set_untracked(VERTEX_SHADER_HANDLE, vertex);

            let fragment = Shader::from_wgsl(include_str!("../assets/fragment.wgsl"))
                .with_import_path(FRAGMENT_SHADER_IMPORT);
            shaders.set_untracked(FRAGMENT_SHADER_HANDLE, fragment);

            let fill = Shader::from_wgsl(include_str!("../assets/fills/cubic_falloff.wgsl"))
                .with_import_path(DEFAULT_FILL_IMPORT);
            shaders.set_untracked(DEFAULT_FILL_HANDLE, fill);
        }
    }
}
