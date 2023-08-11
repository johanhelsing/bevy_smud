#define_import_path smud::gallery::donut

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return abs(smud::sd_circle(p, 18.)) - 3.;
}