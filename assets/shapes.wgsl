fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_renormalize_uv(uv: vec2<f32>) -> vec2<f32> {
    return uv * 2. - vec2<f32>(1., 1.);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothStep(aaf, -aaf, distance);
}

// I think this one looks better than the fwidth version, but perhaps it's more expensive?
fn sd_fill_alpha_dpd(distance: f32) -> f32 {
    let aaf = length(vec2<f32>(dpdx(distance), dpdy(distance))) * 0.71;
    return smoothStep(aaf, -aaf, distance);
}

// Dirt cheap, but ugly
fn sd_fill_alpha_nearest(distance: f32) -> f32 {
    return step(-distance, 0.);
}