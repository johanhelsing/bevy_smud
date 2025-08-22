#define_import_path smud::gallery::egg

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_egg(input.pos, 25., 10.);
}