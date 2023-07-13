#define_import_path bevy_smud::gallery::moon

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_moon(p, 10., 25., 20.);
}