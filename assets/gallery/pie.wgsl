#define_import_path smud::gallery::pie

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_pie(p, smud::sin_cos(0.8), 25.);
}