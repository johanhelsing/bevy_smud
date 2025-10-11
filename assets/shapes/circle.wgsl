#define_import_path smud::shapes::circle

#import smud

// Parametrized circle SDF
// params.x contains the radius
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_circle(input.pos, input.params.x);
}
