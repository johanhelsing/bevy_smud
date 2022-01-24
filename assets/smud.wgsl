#import bevy_smud::prelude
#import bevy_smud::shapes
#import bevy_smud::colorize

struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

// #import bevy_sprite::mesh2d_struct
// [[group(1), binding(0)]]
// var<uniform> mesh: Mesh2d;

// as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    // [[location(1)]] uv: vec2<f32>; // TODO: could be just index?
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] pos: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(
    vertex: Vertex,
    [[builtin(vertex_index)]] i: u32
) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    let x = select(-1., 1., i % 2u == 0u);
    let y = select(-1., 1., (i / 2u) % 2u == 0u);
    let pos = vertex.position + vec3<f32>(x, y, 0.) * 20.;
    // out.clip_position = view.view_proj * mesh.model * vec4<f32>(pos, 0.0, 1.0);
    out.clip_position = view.view_proj * vec4<f32>(pos, 1.0);
    // out.color = vertex.color;
    // out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    out.color = vertex.color;
    out.pos = vec2<f32>(x, y) * 20.;
    // out.pos = vertex.uv * 20.0;
    return out;
}

struct FragmentInput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] pos: vec2<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let d = sd_circle(in.pos, 15.);
    return colorize_normal(d, 0., in.color.rgb);
    // return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    // return in.color * length(in.pos) / 200.0;
}