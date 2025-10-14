#define_import_path smud_editor::template_fill::outline

#import smud

fn fill(input: smud::FillInput) -> vec4<f32> {
    let d_2 = abs(input.distance - 1.) - 1.;
    let a = smud::sd_fill_alpha_fwidth(d_2);
    return vec4<f32>(input.color.rgb, a * input.color.a);
}
