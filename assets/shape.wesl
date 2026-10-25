// Shapes drawn as `Mesh2d` quads: the vertex stage, and the per-instance data
// both stages read through the mesh's tag.

import bevy_sprite_render::mesh2d::functions::{
    get_tag,
    get_world_from_local,
    mesh2d_position_world_to_clip,
};

struct SmudInstance {
    color: vec4<f32>,
    params: vec4<f32>,
    /// Half-extents in `xy`, padding around them in `z`.
    bounds: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<storage> instances: array<SmudInstance>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    /// Position in shape space.
    @location(0) pos: vec2<f32>,
    @location(1) @interpolate(flat) instance_index: u32,
}

fn instance(instance_index: u32) -> SmudInstance {
    return instances[get_tag(instance_index)];
}

@vertex
fn vertex(
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
) -> VertexOutput {
    let data = instance(instance_index);
    // The mesh is a quad from -1 to 1.
    let corner = position.xy * (data.bounds.xy + data.bounds.zz);
    let world_position = get_world_from_local(instance_index) * vec4<f32>(corner, 0., 1.);
    var out: VertexOutput;
    out.position = mesh2d_position_world_to_clip(world_position);
    out.pos = corner;
    out.instance_index = instance_index;
    return out;
}
