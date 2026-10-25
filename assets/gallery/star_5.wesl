fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_star_5_(input.pos, 10., 2.);
}