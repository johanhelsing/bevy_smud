#define_import_path smud::gallery::blobby_cross

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    let s = 20.;
    return (smud::sd_blobby_cross(input.pos / s, 0.7) * s) - 4.;
}