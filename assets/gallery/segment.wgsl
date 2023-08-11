#define_import_path smud::gallery::segment

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_segment(p, vec2<f32>(-13.), vec2<f32>(13.)) - 3.;
}