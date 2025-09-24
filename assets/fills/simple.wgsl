#define_import_path smud::simple_fill

#import smud

fn fill(input: smud::FillInput) -> vec4<f32> {
    let a = smud::sd_fill_alpha_fwidth(input.distance);
    return vec4<f32>(input.color.rgb, a * input.color.a);
}