#define_import_path smud_editor::template_sdf::blobby_cross

#import smud

fn sdf(input: smud::SdfInput) -> f32 {
    let p = input.pos;
    let l = input.params.x + 100.;
    let t = input.params.y + 30.;
    let h = (input.params.z + 70.) / 100.;
    return (smud::sd_blobby_cross(p / l, h) * l) - t;
}
