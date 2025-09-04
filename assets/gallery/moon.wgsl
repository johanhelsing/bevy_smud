#define_import_path smud::gallery::moon

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_moon(input.pos, 10., 25., 20.);
}