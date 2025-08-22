#define_import_path smud::gallery::triangle

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_equilateral_triangle(input.pos, 20.);
}