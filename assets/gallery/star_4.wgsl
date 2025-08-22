#define_import_path smud::gallery::star_4

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_star(input.pos * 0.5, 10., 4, 3.0);
}