#define_import_path smud::bevy_on_fire

#import smud
#import smud::bevy::sd_bevy as sd_bevy

fn sdf(input: smud::SdfInput) -> f32 {
    // return smud::sd_circle(input.pos, 150.0);
    // return smud::bevy::sdf(input);

    let scale = 300.0;
    let bevy = sd_bevy(input.pos / scale) * scale;

    // Mouse position from params (x, y in world coordinates)
    let mouse_pos = input.params.xy;
    let bevy_hole_scale = 100.1;
    let bevy_hole = sd_bevy((input.pos - mouse_pos) / bevy_hole_scale) * bevy_hole_scale - 5.0;

    // Smooth subtract the mouse circle from the bevy shape
    return smud::op_smooth_subtract(bevy_hole, bevy, 20.0);
}
