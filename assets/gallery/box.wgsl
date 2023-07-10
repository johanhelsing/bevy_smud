#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_box(p, vec2<f32>(30., 20.));
}