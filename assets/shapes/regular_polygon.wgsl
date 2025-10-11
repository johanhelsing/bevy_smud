#define_import_path smud::shapes::regular_polygon

#import smud

// Parametrized regular polygon SDF
// params.x contains the radius (circumradius)
// params.y contains the number of sides (as float, should be cast to int)
fn sdf(input: smud::SdfInput) -> f32 {
    let sides = i32(input.params.y);
    return smud::sd_regular_polygon(input.pos, input.params.x, sides);
}
