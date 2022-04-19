struct FragmentInput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] pos: vec2<f32>;
    [[location(2)]] params: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let d = sdf(in.pos, in.params);
    return fill(d, in.color);
    // return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}