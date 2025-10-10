#define_import_path smud::shapes::capsule

#import smud

// Parametrized capsule SDF
// params.x contains the radius
// params.y contains the half_length
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_capsule(input.pos, input.params.x, input.params.y);
}
