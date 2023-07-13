#define_import_path bevy_smud::gallery::star_4

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_star(p * 0.5, 10., 4, 3.0);
}