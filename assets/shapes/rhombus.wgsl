#define_import_path smud::shapes::rhombus

#import smud

// Rhombus SDF using bounds
// Uses input.bounds for the half-diagonals
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_rhombus(input.pos, input.bounds);
}
