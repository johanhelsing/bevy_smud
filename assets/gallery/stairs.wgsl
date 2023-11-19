#define_import_path smud::gallery::stairs

#import smud

fn sdf(p_in: vec2<f32>) -> f32 {
    let s = 5.;
    let p = p_in - vec2<f32>(-20.);
    return smud::sd_stairs(p / s, vec2<f32>(1.), 8.) * s;
}