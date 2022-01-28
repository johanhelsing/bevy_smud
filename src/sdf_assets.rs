use bevy::prelude::*;

pub trait SdfAssets {
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader>;
}

impl SdfAssets for Assets<Shader> {
    fn add_sdf_body<T: Into<String>>(&mut self, sdf: T) -> Handle<Shader> {
        let body = sdf.into();
        let shader = Shader::from_wgsl(format!(
            r#"
#import bevy_smud::shapes
fn sdf(p: vec2<f32>) -> f32 {{
    {body}
}}
"#
        ));
        self.add(shader)
    }
}
