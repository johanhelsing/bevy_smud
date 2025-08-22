#define_import_path smud::gallery::segment

#import smud
#import smud::prelude::SdfInput

fn sdf(input: SdfInput) -> f32 {
    return smud::sd_segment(input.pos, vec2<f32>(-13.), vec2<f32>(13.)) - 3.;
}