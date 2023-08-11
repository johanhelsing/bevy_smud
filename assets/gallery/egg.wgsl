#define_import_path smud::gallery::egg

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_egg(p, 25., 10.);
}