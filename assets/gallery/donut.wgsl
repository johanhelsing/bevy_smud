#define_import_path bevy_smud::gallery::donut

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return abs(shapes::sd_circle(p, 18.)) - 3.;
}