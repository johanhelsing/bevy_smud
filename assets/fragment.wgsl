#define_import_path smud::fragment

#import smud::sdf
#import smud::fill

struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) params: vec4<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let d = sdf::sdf(in.pos, in.params);
    return fill::fill(d, in.color);
}