use bevy::{prelude::*, utils::Uuid};

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
        let id = Uuid::new_v4();
        let shader = Shader::from_wgsl(
            format!(
                r#"
#define_import_path smud::sdf{id}

#import smud

fn sdf(p: vec2<f32>) -> f32 {{
    {body}
}}
"#
            ),
            file!(),
        );
        self.add(shader)
    }

    fn add_fill_body<T: Into<String>>(&mut self, fill: T) -> Handle<Shader> {
        let body = fill.into();
        let id = Uuid::new_v4();
        let shader = Shader::from_wgsl(
            format!(
                r#"
#define_import_path smud::fill{id}

#import smud

fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {{
    {body}
}}
"#
            ),
            file!(),
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
