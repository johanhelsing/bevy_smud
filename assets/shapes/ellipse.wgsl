#define_import_path smud::shapes::ellipse

#import smud

// Ellipse SDF using bounds
// Uses input.bounds for the semi-major and semi-minor axes (half_size.x, half_size.y)
// If the axes are equal (or nearly equal), delegates to circle SDF since sd_ellipse doesn't handle that case
fn sdf(input: smud::SdfInput) -> f32 {
    let a = input.bounds.x;
    let b = input.bounds.y;

    // Use circle SDF when radii are equal (sd_ellipse doesn't handle this case well)
    if (abs(a - b) < 1e-6) {
        return smud::sd_circle(input.pos, a);
    }

    return smud::sd_ellipse(input.pos, a, b);
}
