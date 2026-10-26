fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_segment(input.pos, vec2<f32>(-13.), vec2<f32>(13.)) - 3.;
}