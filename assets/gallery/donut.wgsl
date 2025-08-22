#define_import_path smud::gallery::donut

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return abs(smud::sd_circle(input.pos, 18.)) - 3.;
}