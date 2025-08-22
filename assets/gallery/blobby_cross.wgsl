#define_import_path smud::gallery::blobby_cross

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    let s = 20.;
    return (smud::sd_blobby_cross(input.pos / s, 0.7) * s) - 4.;
}