#define_import_path smud::gallery::heart

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_heart((p / 40.) - vec2<f32>(0., -0.5)) * 40.;
}