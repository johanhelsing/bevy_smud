#define_import_path smud::gallery::ellipse

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_ellipse(input.pos, 25., 15.);
}