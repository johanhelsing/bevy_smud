#define_import_path smud::gallery::triangle

#import smud

fn sdf(p: vec2<f32>) -> f32 {
    return smud::sd_equilateral_triangle(p, 20.);
}