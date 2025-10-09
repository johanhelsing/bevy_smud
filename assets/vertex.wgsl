#define_import_path smud::vertex

struct View {
    view_proj: mat4x4<f32>,
    world_position: vec3<f32>,
};
@group(0) @binding(0)
var<uniform> view: View;

// as specified in `specialize()`
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
    @location(3) rotation: vec2<f32>,
    @location(4) scale: f32,
    @location(5) frame: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) params: vec4<f32>,
};

@vertex
fn vertex(
    vertex: Vertex,
    @builtin(vertex_index) i: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x = select(-1., 1., i % 2u == 0u);
    let y = select(-1., 1., (i / 2u) % 2u == 0u);
    let c = vertex.rotation.x;
    let s = vertex.rotation.y;
    // Scale by frame first to get the rectangle shape
    let corner = vec2<f32>(x, y) * vertex.frame;
    // Then rotate the rectangle
    let rotated = vec2<f32>(corner.x * c - corner.y * s, corner.x * s + corner.y * c);
    // Then apply scale
    let pos = vertex.position + vec3<f32>(rotated * vertex.scale, 0.);
    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * vec4<f32>(pos, 1.);
    out.color = vertex.color;
    out.params = vertex.params;
    out.pos = corner;
    return out;
}