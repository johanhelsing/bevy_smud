#define_import_path smud::gallery::pie

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_pie(input.pos, smud::sin_cos(0.8), 25.);
}