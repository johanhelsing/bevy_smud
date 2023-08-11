#define_import_path smud::gallery::box

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_box(p, vec2<f32>(30., 20.));
}