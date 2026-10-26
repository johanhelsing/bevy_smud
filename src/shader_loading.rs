use std::borrow::Cow;

use crate::util::generate_shader_id;

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    platform::collections::HashMap,
    prelude::*,
    shader::{ShaderImport, Source},
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

/// The WESL import path of a loaded sdf or fill shader, or `None` while it loads.
pub(crate) fn import_path_of(
    shaders: &Assets<Shader>,
    handle: &Handle<Shader>,
    kind: &str,
) -> Option<String> {
    let Some(shader) = shaders.get(handle) else {
        debug!("Waiting for {kind} to load");
        return None;
    };
    if !matches!(shader.source, Source::Wesl(_)) {
        // Only WESL shaders can be imported. A `.wgsl` file is plain WGSL, so
        // the generated shader would wait on it forever.
        warn_once!(
            "smud {kind} shader `{}` is not WESL. Rename it to `.wesl`.",
            shader.path
        );
        return None;
    }
    Some(wesl_import_path(shader))
}

/// The fragment shaders of smud's instanced pipelines (UI shapes, and world
/// shapes on WebGL2), one per sdf and fill pair.
#[derive(Default, Resource)]
pub(crate) struct GeneratedShaders(
    pub(crate) HashMap<(AssetId<Shader>, AssetId<Shader>), Handle<Shader>>,
);

impl GeneratedShaders {
    /// Generate and add the shader for an sdf and fill pair
    pub(crate) fn try_generate(
        &mut self,
        sdf: &Handle<Shader>,
        fill: &Handle<Shader>,
        shaders: &mut Assets<Shader>,
    ) -> Option<Handle<Shader>> {
        let shader_key = (sdf.id(), fill.id());

        if let Some(handle) = self.0.get(&shader_key) {
            return Some(handle.clone());
        }

        let sdf_import_path = import_path_of(shaders, sdf, "sdf")?;
        let fill_import_path = import_path_of(shaders, fill, "fill")?;

        debug!("Generating shader");
        let generated_shader = Shader::from_wesl(
            format!(
                r#"
@if(TONEMAP_IN_SHADER)
import bevy_core_pipeline::tonemapping;
@if(SRGB_OUTPUT)
import bevy_render::color_operations::linear_to_srgb;
@if(OKLAB_OUTPUT)
import bevy_render::color_operations::linear_rgb_to_oklab;

import bevy_smud::view_bindings::view;

import {sdf_import_path} as sdf;
import {fill_import_path} as fill;

struct FragmentInput {{
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) params: vec4<f32>,
    @location(3) bounds: vec2<f32>,
}};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {{
    let sdf_input = smud::SdfInput(in.pos, in.params, in.bounds);
    let d = sdf::sdf(sdf_input);
    let fill_input = smud::FillInput(
        in.pos,
        in.params,
        d,
        in.color,
    );
    var color = fill::fill(fill_input);

@if(TONEMAP_IN_SHADER)
    color = tonemapping::tone_mapping(color, view.color_grading);

@if(SRGB_OUTPUT)
    color = vec4(linear_to_srgb(color.rgb), color.a);

@if(OKLAB_OUTPUT)
    color = vec4(linear_rgb_to_oklab(color.rgb), color.a);

    return color;
}}
"#
            ),
            // An asset path, so `package::` in the imports above names the
            // asset root, where shaders loaded from files live.
            format!("bevy_smud/generated/shape{}.wesl", generate_shader_id()),
        );

        let generated_shader_handle = shaders.add(generated_shader);

        self.0.insert(shader_key, generated_shader_handle.clone());

        Some(generated_shader_handle)
    }
}
