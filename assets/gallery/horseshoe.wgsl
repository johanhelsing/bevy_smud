#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_horseshoe(p, sin_cos(0.4), 17., vec2<f32>(6., 4.));
}