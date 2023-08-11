#define_import_path smud::gallery::blobby_cross

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    let s = 20.;
    return (smud::sd_blobby_cross(p / s, 0.7) * s) - 4.;
}