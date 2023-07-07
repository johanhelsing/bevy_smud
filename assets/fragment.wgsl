#define_import_path bevy_smud::fragment
#import bevy_smud::sdf sdf
#import bevy_smud::fill fill

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let d = sdf(in.pos);
    return fill(d, in.color);
}