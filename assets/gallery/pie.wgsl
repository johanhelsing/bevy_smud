#define_import_path smud::gallery::pie

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_pie(input.pos, smud::sin_cos(0.8), 25.);
}