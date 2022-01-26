#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_horseshoe(p, sin_cos(0.4), 17., vec2<f32>(6., 4.));
}