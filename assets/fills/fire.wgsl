#define_import_path smud::fire_fill

fn fill(input: smud::FillInput) -> vec4<f32> {
    // TODO: get position here for a more interesting fire effect
    // gradient + noise?
    let d2 = 1. - (input.distance * 0.13);
    let alpha = clamp(d2 * d2 * d2, 0., 1.) * input.color.a;
    let shadow_color = 0.2 * input.color.rgb;
    let aaf = 0.7 / fwidth(input.distance);
    let c = vec3<f32>(1.0, -input.pos.y * 0.003, 0.0);
    return vec4<f32>(c, alpha);
}