use bevy::prelude::*;

use crate::{shader_loading::wesl_module, util::generate_shader_id};

/// Extension trait for Assets<Shader> for conveniently creating new shaders from code
pub trait SdfAssets {
    /// Create a sdf shader from the given wgsl body
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader>;
    /// Create a sdf shader from the given wgsl expression
    fn add_sdf_expr<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader>;
    /// Create a fill shader from the given wgsl body
    fn add_fill_body<T: Into<String>>(&mut self, fill: T) -> Handle<Shader>;
    /// Create a fill shader from the given wgsl expression
    fn add_fill_expr<T: Into<String>>(&mut self, fill: T) -> Handle<Shader>;
}

impl SdfAssets for Assets<Shader> {
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader> {
        let body = sdf.into();
        let id = generate_shader_id();
        let shader = wesl_module(
            format!(
                r#"
fn sdf(input: smud::SdfInput) -> f32 {{
    // Backward compatibility aliases for the old API
    let p = input.pos;
    let params = input.params;
    let bounds = input.bounds;

    {body}
}}
"#
            ),
            file!(),
            &format!("smud::sdf{id}"),
        );
        self.add(shader)
    }

    fn add_fill_body<T: Into<String>>(&mut self, fill: T) -> Handle<Shader> {
        let body = fill.into();
        let id = generate_shader_id();
        let shader = wesl_module(
            format!(
                r#"
import bevy_smud::view_bindings::globals;

fn fill(input: smud::FillInput) -> vec4<f32> {{
    let time = globals.time;
    let p = input.pos;
    let params = input.params;
    let d = input.distance;
    let color = input.color;
    {body}
}}
"#
            ),
            file!(),
            &format!("smud::fill{id}"),
        );
        self.add(shader)
    }

    fn add_sdf_expr<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader> {
        let e = sdf.into();
        self.add_sdf_body(format!("return {e};"))
    }

    fn add_fill_expr<T: Into<String>>(&mut self, fill: T) -> Handle<Shader> {
        let e = fill.into();
        self.add_fill_body(format!("return {e};"))
    }
}
