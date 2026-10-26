fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_star(input.pos * 0.5, 10., 4, 3.0);
}