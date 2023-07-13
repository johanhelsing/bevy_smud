#define_import_path bevy_smud::gallery::horseshoe

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_horseshoe(p, shapes::sin_cos(0.4), 17., vec2<f32>(6., 4.));
}