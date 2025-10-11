#define_import_path smud::shapes::rhombus

#import smud

// Parametrized rhombus SDF
// params.x contains half of the horizontal diagonal
// params.y contains half of the vertical diagonal
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_rhombus(input.pos, vec2<f32>(input.params.x, input.params.y));
}
