#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_segment(p, vec2<f32>(-13.), vec2<f32>(13.)) - 3.;
}