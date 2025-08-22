#define_import_path smud::gallery::hexagon

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_hexagon(input.pos, 20.);
}