#define_import_path smud::gallery::heart

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_heart((input.pos / 40.) - vec2<f32>(0., -0.5)) * 40.;
}