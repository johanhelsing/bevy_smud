#define_import_path bevy_smud::gallery::heart

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_heart((p / 40.) - vec2<f32>(0., -0.5)) * 40.;
}