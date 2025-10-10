#define_import_path smud::shapes::annulus

#import smud

// Parametrized annulus (ring) SDF
// params.x contains the outer radius
// params.y contains the inner radius
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_annulus(input.pos, input.params.x, input.params.y);
}
