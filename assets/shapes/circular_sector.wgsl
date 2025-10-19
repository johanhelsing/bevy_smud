#define_import_path smud::shapes::circular_sector

#import smud

// Circular sector (pie slice) SDF using bounds for radius
// Radius is computed as min(bounds.x, bounds.y)
// params.x contains sin(half_angle)
// params.y contains cos(half_angle)
fn sdf(input: smud::SdfInput) -> f32 {
    let radius = min(input.bounds.x, input.bounds.y);
    let c = vec2<f32>(input.params.x, input.params.y);
    return smud::sd_pie(input.pos, c, radius);
}
