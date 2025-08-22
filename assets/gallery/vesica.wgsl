#define_import_path smud::gallery::vesica

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_vesica(input.pos, 30., 15.);
}