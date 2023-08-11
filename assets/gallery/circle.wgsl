#define_import_path smud::gallery::circle

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_circle(p, 25.);
}