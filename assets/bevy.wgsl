#define_import_path smud::bevy

#import smud

fn bevy_head(p: vec2<f32>) -> f32 {
    let skull = smud::sd_ellipse(p, 0.22, 0.20);
    let p_beak = smud::rotate_rad(p - vec2<f32>(0.12, 0.02), 1.2);
    let beak = smud::sd_vesica(p_beak, 0.3, 0.2);
    return min(skull, beak);
}

fn sdf(p_in: vec2<f32>) -> f32 {
    let scale = 300.0;
    var p = p_in / scale;

    let p_upper_wing = p - vec2<f32>(-0.3, -0.25);
    let upper_wing = max(
        smud::sd_ellipse(p_upper_wing, 0.7, 0.6),
        -smud::rotate_rad(p, 0.40).y - 0.03
        // -sd_circle(p_upper_wing - vec2<f32>(-0.35, -0.05), 0.6)
    );
    let p_lower_wing = p - vec2<f32>(-0.3, -0.35);
    let lower_wing = max(
        smud::sd_ellipse(p_lower_wing, 0.7, 0.5),
        -p.y - 0.5
    );

    let wings = max(min(lower_wing, upper_wing), max(-p.y - 0.5, p.x - 0.10));

    let chest_clip = max(-p.y - 0.35, p.x - 0.1);
    let tail_clip = p.x + 0.7;

    let head = bevy_head(p - vec2<f32>(0.18, 0.40));

    let chest = smud::op_smooth_intersect(
        smud::sd_ellipse(p - vec2<f32>(-0.8, -0.05), 1.3, 0.7),
        max(-chest_clip, -tail_clip),
        0.04
        // -sd_ellipse(p - vec2<f32>(-0.8, 0.15), 0.9, 0.8)
    );

    let tail_wing_hole = smud::sd_ellipse(smud::rotate_rad(p -vec2<f32>(-0.8, -0.4), -0.1), 0.63, 0.25);

    let chest_head = smud::op_smooth_union(chest, head, 0.07);
    let chest_head_tail = smud::op_smooth_subtract(tail_wing_hole, chest_head, 0.07);

    let body = smud::op_smooth_union(
        chest_head_tail,
        max(wings, -tail_wing_hole + 0.01),
        0.01
    );

    let eye = smud::sd_circle(p - vec2<f32>(0.20, 0.45), 0.05);
    let bevy = max(body, -eye);

    return bevy * scale;
}