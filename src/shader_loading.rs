use bevy::{
    asset::{load_internal_asset, weak_handle},
    prelude::*,
};

const PRELUDE_SHADER_HANDLE: Handle<Shader> = weak_handle!("33980330-e199-4485-8687-423334e08152");

const SMUD_SHADER_HANDLE: Handle<Shader> = weak_handle!("eca0ed31-c377-41be-94a8-55154ecb0810");

const VIEW_BINDINGS_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("f973016d-a9cc-469a-afb6-64ad829cd838");

pub const VERTEX_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("27b9d87f-6a69-49ee-a2e8-c0bc08ee4f61");

pub const FRAGMENT_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("b42388f6-f32a-488a-b2e1-5819f2aa659e");

/// The default fill used by `SmudShape`
pub const DEFAULT_FILL_HANDLE: Handle<Shader> =
    weak_handle!("30981e86-7600-4089-b4e7-992601dc96b4");

/// Simple single-colored filled fill
pub const SIMPLE_FILL_HANDLE: Handle<Shader> = weak_handle!("cef2d2c2-1a68-4418-a815-5a8ac361f140");

pub struct ShaderLoadingPlugin;

impl Plugin for ShaderLoadingPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PRELUDE_SHADER_HANDLE,
            "../assets/prelude.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SMUD_SHADER_HANDLE,
            "../assets/smud.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            VIEW_BINDINGS_SHADER_HANDLE,
            "../assets/view_bindings.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            VERTEX_SHADER_HANDLE,
            "../assets/vertex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            FRAGMENT_SHADER_HANDLE,
            "../assets/fragment.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            DEFAULT_FILL_HANDLE,
            "../assets/fills/cubic_falloff.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SIMPLE_FILL_HANDLE,
            "../assets/fills/simple.wgsl",
            Shader::from_wgsl
        );
    }
}
