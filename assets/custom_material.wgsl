#import "prelude.wgsl"
#import "shapes.wgsl"
#import "colorize.wgsl"

struct CustomMaterial {
    color: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> material: CustomMaterial;

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
#ifdef VERTEX_TANGENTS
    [[location(3)]] world_tangent: vec4<f32>;
#endif
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let width = 15.;
    let pos = sd_renormalize_uv(in.uv) * width;
    let d = sd_circle(pos, width / 2.);
    return colorize_normal(d, 0., material.color.rgb);
}
