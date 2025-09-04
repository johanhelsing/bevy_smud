#define_import_path smud::gallery::triangle

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_equilateral_triangle(input.pos, 20.);
}