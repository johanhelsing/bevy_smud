#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_segment(p, vec2<f32>(-13.), vec2<f32>(13.)) - 3.;
}