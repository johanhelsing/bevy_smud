#define_import_path smud::gallery::vesica

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_vesica(p, 30., 15.);
}