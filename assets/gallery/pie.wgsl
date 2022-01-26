#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_pie(p, sin_cos(0.8), 25.);
}