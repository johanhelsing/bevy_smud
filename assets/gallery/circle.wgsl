#define_import_path bevy_smud::gallery::circle

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_circle(p, 25.);
}