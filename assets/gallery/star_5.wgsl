#define_import_path smud::gallery::star_5

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_star_5_(input.pos, 10., 2.);
}