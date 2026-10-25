fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_circle(input.pos, 25.);
}