#define_import_path smud::gallery::circle

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_circle(input.pos, 25.);
}