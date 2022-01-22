// Import the standard 2d mesh uniforms and set their bind groups

#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;

#import bevy_sprite::mesh2d_struct
[[group(1), binding(0)]]
var<uniform> mesh: Mesh2d;

// as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = vertex.color;
    return out;
}

struct FragmentInput {
    // The color is interpolated between vertices by default
    [[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    return in.color;
}