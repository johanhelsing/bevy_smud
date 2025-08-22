#define_import_path smud::gallery::box

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_box(input.pos, vec2<f32>(30., 20.));
}