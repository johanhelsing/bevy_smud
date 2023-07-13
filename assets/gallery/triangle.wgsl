#define_import_path bevy_smud::gallery::triangle

#import bevy_smud::shapes as shapes

fn sdf(p: vec2<f32>) -> f32 {
    return shapes::sd_equilateral_triangle(p, 20.);
}