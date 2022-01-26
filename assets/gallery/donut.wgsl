#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return abs(sd_circle(p, 18.)) - 3.;
}