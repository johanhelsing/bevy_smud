#define_import_path bevy_smud::gallery::ellipse

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_ellipse(p, 25., 15.);
}