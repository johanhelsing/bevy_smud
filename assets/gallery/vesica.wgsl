#define_import_path smud::gallery::vesica

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_vesica(input.pos, 30., 15.);
}