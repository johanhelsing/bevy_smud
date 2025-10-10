#define_import_path smud::shapes::ellipse

#import smud

// Parametrized ellipse SDF
// params.x contains the semi-major axis (half_size.x)
// params.y contains the semi-minor axis (half_size.y)
// If the axes are equal (or nearly equal), delegates to circle SDF since sd_ellipse doesn't handle that case
fn sdf(input: smud::SdfInput) -> f32 {
    let a = input.params.x;
    let b = input.params.y;

    // Use circle SDF when radii are equal (sd_ellipse doesn't handle this case well)
    if (abs(a - b) < 1e-6) {
        return smud::sd_circle(input.pos, a);
    }

    return smud::sd_ellipse(input.pos, a, b);
}
