use bevy::prelude::*;

pub trait SdfAssets {
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader>;
    fn add_sdf_expr<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader>;
    fn add_fill_body<T: Into<String>>(&mut self, fill: T) -> Handle<Shader>;
    fn add_fill_expr<T: Into<String>>(&mut self, fill: T) -> Handle<Shader>;
}

impl SdfAssets for Assets<Shader> {
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader> {
        let body = sdf.into();
        let shader = Shader::from_wgsl(format!(
            r#"
#import bevy_smud::shapes
fn sdf(p: vec2<f32>, params: vec4<f32>) -> f32 {{
    {body}
}}
"#
        ));
        self.add(shader)
    }

    fn add_fill_body<T: Into<String>>(&mut self, fill: T) -> Handle<Shader> {
        let body = fill.into();
        let shader = Shader::from_wgsl(format!(
            r#"
fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {{
    {body}
}}
"#
        ));
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
