use bevy::{prelude::*, reflect::TypeUuid};

const PRELUDE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11291576006157771079);
const PRELUDE_SHADER_IMPORT: &str = "bevy_smud::prelude";

const SHAPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10055894596049459186);
const SHAPES_SHADER_IMPORT: &str = "bevy_smud::shapes";

pub const VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16846632126033267571);
const VERTEX_SHADER_IMPORT: &str = "bevy_smud::vertex";

pub const FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10370213491934870425);
const FRAGMENT_SHADER_IMPORT: &str = "bevy_smud::fragment";

/// The default fill used by `SmudShape`
pub const DEFAULT_FILL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 18184663565780163454);
const DEFAULT_FILL_IMPORT: &str = "bevy_smud::default_fill";

/// Simple single-colored filled fill
pub const SIMPLE_FILL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 16286090377316294491);
const SIMPLE_FILL_IMPORT: &str = "bevy_smud::simple_fill";

pub struct ShaderLoadingPlugin;

impl Plugin for ShaderLoadingPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

        let prelude = Shader::from_wgsl(include_str!("../assets/prelude.wgsl"), "a")
            .with_import_path(PRELUDE_SHADER_IMPORT);
        shaders.set_untracked(PRELUDE_SHADER_HANDLE, prelude);

        let shapes = Shader::from_wgsl(include_str!("../assets/shapes.wgsl"), "b")
            .with_import_path(SHAPES_SHADER_IMPORT);
        shaders.set_untracked(SHAPES_SHADER_HANDLE, shapes);

        let vertex = Shader::from_wgsl(include_str!("../assets/vertex.wgsl"), "c")
            .with_import_path(VERTEX_SHADER_IMPORT);
        shaders.set_untracked(VERTEX_SHADER_HANDLE, vertex);

        let fragment = Shader::from_wgsl(include_str!("../assets/fragment.wgsl"), "d")
            .with_import_path(FRAGMENT_SHADER_IMPORT);
        shaders.set_untracked(FRAGMENT_SHADER_HANDLE, fragment);

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let fill = Shader::from_wgsl(include_str!("../assets/fills/cubic_falloff.wgsl"), "e")
            .with_import_path(DEFAULT_FILL_IMPORT);
        shaders.set_untracked(DEFAULT_FILL_HANDLE, fill);

        let simple_fill = Shader::from_wgsl(include_str!("../assets/fills/simple.wgsl"), "f")
            .with_import_path(SIMPLE_FILL_IMPORT);
        shaders.set_untracked(SIMPLE_FILL_HANDLE, simple_fill);
    }
}
