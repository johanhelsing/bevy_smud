#define_import_path bevy_smud::gallery::pie

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_pie(p, shapes::sin_cos(0.8), 25.);
}