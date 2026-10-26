// Rectangle SDF using bounds
// Uses input.bounds for the half-size of the rectangle
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_box(input.pos, input.bounds);
}
