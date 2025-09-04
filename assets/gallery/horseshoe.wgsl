#define_import_path smud::gallery::horseshoe

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_horseshoe(input.pos, smud::sin_cos(0.4), 17., vec2<f32>(6., 4.));
}