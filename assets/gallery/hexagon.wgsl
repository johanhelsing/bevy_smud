#define_import_path bevy_smud::gallery::hexagon

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_hexagon(p, 20.);
}