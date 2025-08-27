#define_import_path smud::default_fill

#import smud

fn fill(input: smud::FillInput) -> vec4<f32> {
    let d2 = 1. - (input.distance * 0.13);
    let alpha = clamp(d2 * d2 * d2, 0., 1.) * input.color.a;
    let shadow_color = 0.2 * input.color.rgb;
    let aaf = 0.7 / fwidth(input.distance);
    let c = mix(input.color.rgb, shadow_color, clamp(input.distance * aaf, 0., 1.));
    return vec4<f32>(c, alpha);
}