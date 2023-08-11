#define_import_path smud::default_fill

fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {
    let d2 = 1. - (d * 0.13);
    let alpha = clamp(d2 * d2 * d2, 0., 1.) * color.a;
    let shadow_color = 0.2 * color.rgb;
    let aaf = 0.7 / fwidth(d);
    let c = mix(color.rgb, shadow_color, clamp(d * aaf, 0., 1.));
    return vec4<f32>(c, alpha);
}