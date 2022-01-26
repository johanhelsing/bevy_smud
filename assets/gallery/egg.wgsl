#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_egg(p, 25., 10.);
}