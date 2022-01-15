fn iq_palette(t: f32, a: float3, b: float3, c: float3, d: float3) -> float3 {
    return a + b * cos(2. * PI * (c * t + d));
}

fn colorize_normal(d: f32, travel: f32, color: float3) -> float4 {
    let d2 = 1. - (d * 0.13);
    let alpha = d2 * d2 * d2;
    let shadow_color = 0.2 * color;
    let aaf = 0.7 / fwidth(d);
    let c = mix(color, shadow_color, clamp(d * aaf, 0., 1.));
    return float4(c, alpha);
}