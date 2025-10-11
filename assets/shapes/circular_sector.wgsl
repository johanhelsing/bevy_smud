#define_import_path smud::shapes::circular_sector

#import smud

// Parametrized circular sector (pie slice) SDF
// params.x contains the radius
// params.y contains sin(half_angle)
// params.z contains cos(half_angle)
fn sdf(input: smud::SdfInput) -> f32 {
    let c = vec2<f32>(input.params.y, input.params.z);
    return smud::sd_pie(input.pos, c, input.params.x);
}
