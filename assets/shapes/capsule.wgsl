#define_import_path smud::shapes::capsule

#import smud

// Capsule SDF using bounds
// Radius is min(bounds.x, bounds.y) to ensure half_length remains positive
// Half-length of the line segment is bounds.y - radius
fn sdf(input: smud::SdfInput) -> f32 {
    let radius = min(input.bounds.x, input.bounds.y);
    let half_length = input.bounds.y - radius;
    return smud::sd_capsule(input.pos, radius, half_length);
}
