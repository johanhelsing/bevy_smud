#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_heart((p / 40.) - vec2<f32>(0., -0.5)) * 40.;
}