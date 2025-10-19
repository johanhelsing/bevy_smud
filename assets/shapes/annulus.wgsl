#define_import_path smud::shapes::annulus

#import smud

// Annulus (ring) SDF using bounds for outer radius
// Outer radius is computed as min(bounds.x, bounds.y)
// params.x contains the inner radius
fn sdf(input: smud::SdfInput) -> f32 {
    let outer_radius = min(input.bounds.x, input.bounds.y);
    let inner_radius = input.params.x;
    return smud::sd_annulus(input.pos, outer_radius, inner_radius);
}
