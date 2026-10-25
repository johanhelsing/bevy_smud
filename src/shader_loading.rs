use std::borrow::Cow;

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    prelude::*,
    shader::ShaderImport,
};

const SMUD_SHADER_HANDLE: Handle<Shader> = uuid_handle!("eca0ed31-c377-41be-94a8-55154ecb0810");

const VIEW_BINDINGS_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("f973016d-a9cc-469a-afb6-64ad829cd838");

pub const VERTEX_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("27b9d87f-6a69-49ee-a2e8-c0bc08ee4f61");

/// The default fill used by `SmudShape`
pub const DEFAULT_FILL_HANDLE: Handle<Shader> =
    uuid_handle!("30981e86-7600-4089-b4e7-992601dc96b4");

/// Simple single-colored filled fill
pub const SIMPLE_FILL_HANDLE: Handle<Shader> = uuid_handle!("cef2d2c2-1a68-4418-a815-5a8ac361f140");

pub struct ShaderLoadingPlugin;

impl Plugin for ShaderLoadingPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SMUD_SHADER_HANDLE,
            "../assets/smud.wesl",
            wesl_module,
            "smud"
        );

        load_internal_asset!(
            app,
            VIEW_BINDINGS_SHADER_HANDLE,
            "../assets/view_bindings.wesl",
            wesl_module,
            "bevy_smud::view_bindings"
        );

        load_internal_asset!(
            app,
            VERTEX_SHADER_HANDLE,
            "../assets/vertex.wesl",
            wesl_module,
            "bevy_smud::vertex"
        );

        load_internal_asset!(
            app,
            DEFAULT_FILL_HANDLE,
            "../assets/fills/cubic_falloff.wesl",
            wesl_module,
            "smud::default_fill"
        );

        load_internal_asset!(
            app,
            SIMPLE_FILL_HANDLE,
            "../assets/fills/simple.wesl",
            wesl_module,
            "smud::simple_fill"
        );
    }
}

/// A WESL shader that other shaders import by `module`, such as `smud::default_fill`.
///
/// WESL names a module after its asset path. smud's own shaders are not loaded
/// from a path, so they get a fixed module path instead.
pub(crate) fn wesl_module(
    source: impl Into<Cow<'static, str>>,
    path: impl Into<String>,
    module: &str,
) -> Shader {
    let mut shader = Shader::from_wesl(source, path);
    shader.import_path = ShaderImport::Custom(module.into());
    shader
}

/// How a WESL shader imports the module of `shader`.
///
/// A shader loaded from an asset is imported by its asset path relative to the
/// asset root: `package::gallery::heart` for `gallery/heart.wesl`.
pub(crate) fn wesl_import_path(shader: &Shader) -> String {
    match &shader.import_path {
        ShaderImport::Custom(module) => module.clone(),
        ShaderImport::AssetPath(path) => std::iter::once("package")
            .chain(path.split('/').filter(|component| !component.is_empty()))
            .collect::<Vec<_>>()
            .join("::"),
    }
}
