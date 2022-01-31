#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_equilateral_triangle(p, 20.);
}