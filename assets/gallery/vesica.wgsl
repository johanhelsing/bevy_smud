#define_import_path bevy_smud::gallery::vesica

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_vesica(p, 30., 15.);
}