#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_moon(p, 10., 25., 20.);
}