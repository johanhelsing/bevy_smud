fn fill(d: f32, color: vec4<f32>) -> vec4<f32> {
    let d_2 = abs(d - 1.) - 1.;
    let a = sd_fill_alpha_fwidth(d_2);
    return vec4<f32>(color.rgb, a * color.a);
}