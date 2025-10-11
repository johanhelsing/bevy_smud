#define_import_path smud::shapes::rectangle

#import smud

// Parametrized rectangle SDF
// params.xy contains the half-size of the rectangle
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_box(input.pos, input.params.xy);
}
