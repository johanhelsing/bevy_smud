#define_import_path smud::simple_fill

#import smud

fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {
    let a = smud::sd_fill_alpha_fwidth(d);
    return vec4<f32>(color.rgb, a * color.a);
}