use bevy::{asset::load_internal_asset, prelude::*};

const PRELUDE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(11291576006157771079);

const SMUD_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(10055894596049459186);

const VIEW_BINDINGS_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(11792080578571156967);

pub const VERTEX_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(16846632126033267571);

pub const FRAGMENT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(10370213491934870425);

/// The default fill used by `SmudShape`
pub const DEFAULT_FILL_HANDLE: Handle<Shader> = Handle::weak_from_u128(18184663565780163454);

/// Simple single-colored filled fill
pub const SIMPLE_FILL_HANDLE: Handle<Shader> = Handle::weak_from_u128(16286090377316294491);

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
