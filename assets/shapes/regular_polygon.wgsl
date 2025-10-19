#define_import_path smud::shapes::regular_polygon

#import smud

// Regular polygon SDF using bounds for radius
// Radius is computed as min(bounds.x, bounds.y)
// params.x contains the number of sides (as float, should be cast to int)
fn sdf(input: smud::SdfInput) -> f32 {
    let radius = min(input.bounds.x, input.bounds.y);
    let sides = i32(input.params.x);
    return smud::sd_regular_polygon(input.pos, radius, sides);
}
