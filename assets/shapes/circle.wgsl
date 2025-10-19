#define_import_path smud::shapes::circle

#import smud

// Circle SDF using bounds
// Radius is computed as min(bounds.x, bounds.y)
fn sdf(input: smud::SdfInput) -> f32 {
    let radius = min(input.bounds.x, input.bounds.y);
    return smud::sd_circle(input.pos, radius);
}
