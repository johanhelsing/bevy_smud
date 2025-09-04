//! Rust implementations of SDF (Signed Distance Field) functions.
//!
//! These functions mirror the WGSL SDF functions available in the shader,
//! allowing for precise picking and other CPU-side SDF calculations.
//!
//! All functions take a position `Vec2` and return the signed distance to the shape surface.
//! Negative values indicate the point is inside the shape, positive values indicate outside,
//! and zero indicates the point is on the surface.

use bevy::prelude::*;

/// Circle SDF: distance from center minus radius
/// 
/// # Arguments
/// * `p` - Point to test
/// * `radius` - Radius of the circle
/// 
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_smud::sdf;
/// 
/// let distance = sdf::circle(Vec2::new(50.0, 0.0), 100.0);
/// assert!(distance < 0.0); // Point is inside the circle
/// ```
pub fn circle(p: Vec2, radius: f32) -> f32 {
    p.length() - radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_center() {
        let distance = circle(Vec2::ZERO, 10.0);
        assert_eq!(distance, -10.0); // At center, distance is -radius
    }

    #[test]
    fn test_circle_on_edge() {
        let distance = circle(Vec2::new(10.0, 0.0), 10.0);
        assert!((distance - 0.0).abs() < f32::EPSILON); // On edge, distance is 0
    }

    #[test]
    fn test_circle_outside() {
        let distance = circle(Vec2::new(15.0, 0.0), 10.0);
        assert_eq!(distance, 5.0); // Outside by 5 units
    }
}
