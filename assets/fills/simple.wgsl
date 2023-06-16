#define_import_path bevy_smud::simple_fill

#import bevy_smud::shapes as shapes

fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {
    let a = shapes::sd_fill_alpha_fwidth(d);
    return vec4<f32>(color.rgb, a * color.a);
}