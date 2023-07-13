#define_import_path bevy_smud::gallery::star_5

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_star_5_(p, 10., 2.);
}