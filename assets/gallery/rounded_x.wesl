fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_rounded_x(input.pos, 30., 4.);
}