#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    return sd_box(p, vec2<f32>(30., 20.));
}