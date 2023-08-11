#define_import_path smud::gallery::moon

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_moon(p, 10., 25., 20.);
}