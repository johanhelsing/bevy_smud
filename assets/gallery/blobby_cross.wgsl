#import bevy_smud::shapes

fn sdf(p: vec2<f32>) -> f32 {
    let s = 20.;
    return (sd_blobby_cross(p / s, 0.7) * s) - 4.;
}