#define_import_path smud::gallery::ellipse

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_ellipse(input.pos, 25., 15.);
}