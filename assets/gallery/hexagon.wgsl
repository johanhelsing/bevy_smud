#define_import_path smud::gallery::hexagon

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_hexagon(p, 20.);
}