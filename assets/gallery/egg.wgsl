#define_import_path bevy_smud::gallery::egg

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_egg(p, 25., 10.);
}