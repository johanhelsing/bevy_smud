fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_hexagon(input.pos, 20.);
}