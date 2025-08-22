#define_import_path smud::gallery::moon

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_moon(input.pos, 10., 25., 20.);
}