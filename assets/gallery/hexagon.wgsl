#define_import_path smud::gallery::hexagon

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_hexagon(input.pos, 20.);
}