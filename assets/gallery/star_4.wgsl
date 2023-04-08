#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_star(p * 0.5, 10., 4, 3.0);
}