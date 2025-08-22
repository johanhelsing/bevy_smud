#define_import_path smud::gallery::star_4

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_star(input.pos * 0.5, 10., 4, 3.0);
}