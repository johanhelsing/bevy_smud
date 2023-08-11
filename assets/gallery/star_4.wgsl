#define_import_path smud::gallery::star_4

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_star(p * 0.5, 10., 4, 3.0);
}