#define_import_path smud::gallery::star_5

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_star_5_(p, 10., 2.);
}