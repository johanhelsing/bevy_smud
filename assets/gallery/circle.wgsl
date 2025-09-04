#define_import_path smud::gallery::circle

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_circle(input.pos, 25.);
}