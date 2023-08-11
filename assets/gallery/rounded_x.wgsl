#define_import_path smud::gallery::horseshoe

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_rounded_x(p, 30., 4.);
}