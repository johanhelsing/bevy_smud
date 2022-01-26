#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    let s = 5.;
    let p = p - vec2<f32>(-20.);
    return sd_stairs(p / s, vec2<f32>(1.), 8.) * s;
}