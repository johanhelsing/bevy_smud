#define_import_path smud::gallery::horseshoe

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_rounded_x(input.pos, 30., 4.);
}