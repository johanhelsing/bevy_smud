#define_import_path smud::gallery::ellipse

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_ellipse(p, 25., 15.);
}