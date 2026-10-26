fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_egg(input.pos, 25., 10.);
}