//! Convenience conversions from Bevy's math primitives to bevy_smud shapes.
//!
//! This module provides `From` trait implementations that allow you to create
//! `SmudShape` components directly from Bevy's 2D primitive shapes like [`Rectangle`],
//! [`Circle`], [`Ellipse`], and [`Annulus`].
//!
//! # Example
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_smud::prelude::*;
//! # let mut commands: Commands = panic!();
//! commands.spawn((
//!     Transform::from_translation(Vec3::new(100., 0., 0.)),
//!     SmudShape::from(Rectangle::new(100., 50.))
//!         .with_color(Color::srgb(0.8, 0.2, 0.2)),
//! ));
//!
//! commands.spawn((
//!     Transform::from_translation(Vec3::new(-100., 0., 0.)),
//!     SmudShape::from(Circle::new(50.))
//!         .with_color(Color::srgb(0.2, 0.8, 0.2)),
//! ));
//!
//! commands.spawn((
//!     Transform::from_translation(Vec3::new(0., -100., 0.)),
//!     SmudShape::from(Ellipse::new(80., 40.))
//!         .with_color(Color::srgb(0.8, 0.8, 0.2)),
//! ));
//!
//! commands.spawn((
//!     Transform::from_translation(Vec3::new(200., 0., 0.)),
//!     SmudShape::from(Annulus::new(20., 40.))
//!         .with_color(Color::srgb(0.2, 0.8, 0.8)),
//! ));
//! ```
//!
//! # Picking Support
//!
//! When the `bevy_picking` feature is enabled, `SmudPickingShape` is automatically
//! added to entities with primitive-based shapes for precise hit-testing.

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::math::bounding::Bounded2d;
use bevy::math::primitives::{
    Annulus, Capsule2d, Circle, CircularSector, Ellipse, Rectangle, RegularPolygon, Rhombus,
};
use bevy::prelude::*;

use crate::SmudShape;

#[cfg(feature = "bevy_picking")]
use crate::{picking_backend::SdfInput, sdf};

/// Trait for primitives that can be converted to SmudShape.
///
/// This trait encapsulates the varying parts of primitive-to-shape conversion:
/// shader handles, parameter extraction, bounds calculation, and picking functions.
trait SmudPrimitive: Sized + Bounded2d {
    /// The shader handle for this primitive's SDF
    fn sdf_shader() -> Handle<Shader>;

    /// Extract bounds for rendering
    ///
    /// Default implementation uses the `Bounded2d` trait to compute an AABB.
    /// Padding for anti-aliasing is handled by `SmudShape::extra_bounds`.
    fn bounds(&self) -> Rectangle {
        let aabb = Bounded2d::aabb_2d(self, Vec2::ZERO);

        // For asymmetric shapes, we need the maximum extent from origin in each direction
        let half_size = aabb.min.abs().max(aabb.max.abs());

        Rectangle { half_size }
    }

    /// Extract shader parameters (stored in SmudShape.params)
    /// Default implementation returns Vec4::ZERO (no parameters)
    fn params(&self) -> Vec4 {
        Vec4::ZERO
    }

    /// Try to reconstruct a primitive from a SmudShape
    fn try_from_shape(shape: &SmudShape) -> Option<Self>;

    #[cfg(feature = "bevy_picking")]
    /// Create a picking closure for this primitive.
    /// The function receives SdfInput with current position, bounds, and params.
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync>;

    #[cfg(feature = "bevy_picking")]
    /// Try to create a picking shape by reconstructing this primitive from a SmudShape
    fn picking_from_shape(shape: &SmudShape) -> Option<crate::picking_backend::SmudPickingShape> {
        Self::try_from_shape(shape).map(|p| crate::picking_backend::SmudPickingShape::from(p))
    }
}

/// Parametrized rectangle shape SDF
pub const RECTANGLE_SDF_HANDLE: Handle<Shader> =
    uuid_handle!("2289ee84-18da-4e35-87b2-e256fd88c092");

/// Parametrized circle shape SDF
pub const CIRCLE_SDF_HANDLE: Handle<Shader> = uuid_handle!("abb54e5e-62f3-4ea2-9604-84368bb6ae6d");

/// Parametrized ellipse shape SDF
pub const ELLIPSE_SDF_HANDLE: Handle<Shader> = uuid_handle!("2c02adad-84fb-46d7-8ef8-f4b6d86d6149");

/// Parametrized annulus (ring) shape SDF
pub const ANNULUS_SDF_HANDLE: Handle<Shader> = uuid_handle!("a4e4cc45-0af7-4918-b082-69ba5236c4d0");

/// Parametrized capsule (pill) shape SDF
pub const CAPSULE_SDF_HANDLE: Handle<Shader> = uuid_handle!("3f8b7c1d-9e5a-4b2c-8d6f-1a9c4e7b2d5a");

/// Parametrized rhombus shape SDF
pub const RHOMBUS_SDF_HANDLE: Handle<Shader> = uuid_handle!("b41cabff-98bb-417c-92e6-b4889a9290ad");

/// Parametrized circular sector (pie slice) shape SDF
pub const CIRCULAR_SECTOR_SDF_HANDLE: Handle<Shader> =
    uuid_handle!("8c5373ba-2cdc-4e8f-987c-cf5dfd6d84d5");

/// Parametrized regular polygon shape SDF
pub const REGULAR_POLYGON_SDF_HANDLE: Handle<Shader> =
    uuid_handle!("38dc4249-e998-4a6f-ace5-c619ae875929");

/// Plugin that adds support for Bevy primitive shapes.
///
/// This plugin:
/// - Loads shader assets for primitive shapes (Rectangle, Circle, etc.)
/// - Registers observers for auto-adding picking support (when `bevy_picking` feature is enabled)
///
/// This plugin is automatically added by `SmudPlugin`, so you don't need to add it manually.
#[derive(Default)]
pub struct BevyPrimitivesPlugin;

impl Plugin for BevyPrimitivesPlugin {
    fn build(&self, app: &mut App) {
        // Load all primitive shape shaders
        load_internal_asset!(
            app,
            RECTANGLE_SDF_HANDLE,
            "../assets/shapes/rectangle.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CIRCLE_SDF_HANDLE,
            "../assets/shapes/circle.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            ELLIPSE_SDF_HANDLE,
            "../assets/shapes/ellipse.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            ANNULUS_SDF_HANDLE,
            "../assets/shapes/annulus.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CAPSULE_SDF_HANDLE,
            "../assets/shapes/capsule.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            RHOMBUS_SDF_HANDLE,
            "../assets/shapes/rhombus.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CIRCULAR_SECTOR_SDF_HANDLE,
            "../assets/shapes/circular_sector.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            REGULAR_POLYGON_SDF_HANDLE,
            "../assets/shapes/regular_polygon.wgsl",
            Shader::from_wgsl
        );

        // Register observers for auto-adding picking shapes
        #[cfg(feature = "bevy_picking")]
        app.add_observer(auto_add_picking_shape);
    }
}

impl SmudPrimitive for Rectangle {
    fn sdf_shader() -> Handle<Shader> {
        RECTANGLE_SDF_HANDLE
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == RECTANGLE_SDF_HANDLE.id() {
            Some(Rectangle {
                half_size: shape.bounds.half_size,
            })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        Box::new(move |input| sdf::sd_box(input.pos, input.bounds))
    }
}

impl SmudPrimitive for Circle {
    fn sdf_shader() -> Handle<Shader> {
        CIRCLE_SDF_HANDLE
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == CIRCLE_SDF_HANDLE.id() {
            let radius = shape.bounds.half_size.x.min(shape.bounds.half_size.y);
            Some(Circle { radius })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // Circle uses min(bounds.x, bounds.y) for radius in shader
        Box::new(move |input| {
            let radius = input.bounds.x.min(input.bounds.y);
            sdf::circle(input.pos, radius)
        })
    }
}

impl SmudPrimitive for Ellipse {
    fn sdf_shader() -> Handle<Shader> {
        ELLIPSE_SDF_HANDLE
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == ELLIPSE_SDF_HANDLE.id() {
            Some(Ellipse {
                half_size: shape.bounds.half_size,
            })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // Ellipse uses bounds directly for half-extents
        Box::new(move |input| {
            let a = input.bounds.x;
            let b = input.bounds.y;
            const EPSILON: f32 = 1e-6;
            if (a - b).abs() < EPSILON {
                sdf::circle(input.pos, a)
            } else {
                sdf::ellipse(input.pos, a, b)
            }
        })
    }
}

impl SmudPrimitive for Annulus {
    fn sdf_shader() -> Handle<Shader> {
        ANNULUS_SDF_HANDLE
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.inner_circle.radius, 0.0, 0.0, 0.0)
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == ANNULUS_SDF_HANDLE.id() {
            let outer_radius = shape.bounds.half_size.x.min(shape.bounds.half_size.y);
            let inner_radius = shape.params.x;
            Some(Annulus::new(inner_radius, outer_radius))
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // Annulus uses min(bounds.x, bounds.y) for outer radius, inner radius from params
        Box::new(move |input| {
            let outer_radius = input.bounds.x.min(input.bounds.y);
            let inner_radius = input.params.x;
            sdf::annulus(input.pos, outer_radius, inner_radius)
        })
    }
}

impl SmudPrimitive for Capsule2d {
    fn sdf_shader() -> Handle<Shader> {
        CAPSULE_SDF_HANDLE
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == CAPSULE_SDF_HANDLE.id() {
            // Must match shader logic: radius = min(bounds.x, bounds.y)
            let radius = shape.bounds.half_size.x.min(shape.bounds.half_size.y);
            let half_length_from_bounds = shape.bounds.half_size.y - radius;
            // Note: Capsule2d::new takes (radius, full_length), not (radius, half_length)!
            Some(Capsule2d::new(radius, half_length_from_bounds * 2.0))
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // The shader computes from bounds: radius = min(bounds.x, bounds.y), half_length = bounds.y - radius
        Box::new(move |input| {
            let radius = input.bounds.x.min(input.bounds.y);
            let half_length = input.bounds.y - radius;
            sdf::capsule(input.pos, radius, half_length)
        })
    }
}

impl SmudPrimitive for Rhombus {
    fn sdf_shader() -> Handle<Shader> {
        RHOMBUS_SDF_HANDLE
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == RHOMBUS_SDF_HANDLE.id() {
            Some(Rhombus {
                half_diagonals: shape.bounds.half_size,
            })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // Rhombus uses bounds directly for half-diagonals
        Box::new(move |input| sdf::rhombus(input.pos, input.bounds))
    }
}

impl SmudPrimitive for CircularSector {
    fn sdf_shader() -> Handle<Shader> {
        CIRCULAR_SECTOR_SDF_HANDLE
    }

    fn bounds(&self) -> Rectangle {
        // CircularSector uses min(bounds.x, bounds.y) for radius in shader
        // So we need square bounds where both half-extents equal the radius
        Rectangle {
            half_size: Vec2::splat(self.arc.radius),
        }
    }

    fn params(&self) -> Vec4 {
        let (sin, cos) = self.arc.half_angle.sin_cos();
        Vec4::new(sin, cos, 0.0, 0.0)
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == CIRCULAR_SECTOR_SDF_HANDLE.id() {
            let radius = shape.bounds.half_size.x.min(shape.bounds.half_size.y);
            let half_angle = shape.params.x.atan2(shape.params.y);
            Some(CircularSector::new(radius, half_angle))
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // CircularSector uses min(bounds.x, bounds.y) for radius, angle from params
        Box::new(move |input| {
            let radius = input.bounds.x.min(input.bounds.y);
            let c = Vec2::new(input.params.x, input.params.y); // sin, cos
            sdf::pie(input.pos, c, radius)
        })
    }
}

impl SmudPrimitive for RegularPolygon {
    fn sdf_shader() -> Handle<Shader> {
        REGULAR_POLYGON_SDF_HANDLE
    }

    fn bounds(&self) -> Rectangle {
        // RegularPolygon uses min(bounds.x, bounds.y) for radius in shader
        // So we need square bounds where both half-extents equal the circumradius
        Rectangle {
            half_size: Vec2::splat(self.circumcircle.radius),
        }
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.sides as f32, 0.0, 0.0, 0.0)
    }

    fn try_from_shape(shape: &SmudShape) -> Option<Self> {
        if shape.sdf.id() == REGULAR_POLYGON_SDF_HANDLE.id() {
            let radius = shape.bounds.half_size.x.min(shape.bounds.half_size.y);
            let sides = shape.params.x as u32;
            Some(RegularPolygon::new(radius, sides))
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(SdfInput) -> f32 + Send + Sync> {
        // RegularPolygon uses min(bounds.x, bounds.y) for radius, sides from params
        Box::new(move |input| {
            let radius = input.bounds.x.min(input.bounds.y);
            let sides = input.params.x as i32;
            sdf::regular_polygon(input.pos, radius, sides)
        })
    }
}

impl<T: SmudPrimitive> From<T> for SmudShape {
    fn from(primitive: T) -> Self {
        Self {
            sdf: T::sdf_shader(),
            bounds: primitive.bounds(),
            params: primitive.params(),
            ..default()
        }
    }
}

#[cfg(feature = "bevy_picking")]
impl<T: SmudPrimitive> From<T> for crate::picking_backend::SmudPickingShape {
    fn from(primitive: T) -> Self {
        Self::with_input(primitive.picking_fn())
    }
}

/// Observer that automatically adds SmudPickingShape for shapes created from primitives
#[cfg(feature = "bevy_picking")]
fn auto_add_picking_shape(
    trigger: On<Add, SmudShape>,
    query: Query<&SmudShape>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if let Ok(shape) = query.get(entity) {
        // Try to reconstruct the primitive and use its picking function
        let picking_shape = Rectangle::picking_from_shape(shape)
            .or_else(|| Circle::picking_from_shape(shape))
            .or_else(|| Ellipse::picking_from_shape(shape))
            .or_else(|| Annulus::picking_from_shape(shape))
            .or_else(|| Capsule2d::picking_from_shape(shape))
            .or_else(|| Rhombus::picking_from_shape(shape))
            .or_else(|| CircularSector::picking_from_shape(shape))
            .or_else(|| RegularPolygon::picking_from_shape(shape));

        if let Some(picking_shape) = picking_shape {
            commands.entity(entity).insert(picking_shape);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_round_trip() {
        let original = Rectangle::new(100.0, 50.0);
        let shape = SmudShape::from(original);
        let reconstructed =
            Rectangle::try_from_shape(&shape).expect("Failed to reconstruct rectangle");

        assert_eq!(
            original.half_size, reconstructed.half_size,
            "Rectangle half_size should match after round-trip conversion"
        );
    }

    #[test]
    fn test_capsule2d_round_trip() {
        let original = Capsule2d::new(10.0, 20.0);
        let shape = SmudShape::from(original);

        println!(
            "Original: radius={}, half_length={}",
            original.radius, original.half_length
        );
        println!("Original bounds: {:?}", original.bounds());
        println!("Shape bounds: {:?}", shape.bounds.half_size);

        let reconstructed =
            Capsule2d::try_from_shape(&shape).expect("Failed to reconstruct capsule");

        println!(
            "Reconstructed: radius={}, half_length={}",
            reconstructed.radius, reconstructed.half_length
        );

        assert_eq!(
            original.radius, reconstructed.radius,
            "Capsule2d radius should match after round-trip conversion"
        );
        assert_eq!(
            original.half_length, reconstructed.half_length,
            "Capsule2d half_length should match after round-trip conversion"
        );
    }

    #[test]
    fn test_capsule2d_with_wide_bounds() {
        // Test a capsule with bounds wider than tall (70x40)
        let original = Capsule2d::new(10.0, 20.0);
        let mut shape = SmudShape::from(original);
        shape.bounds = Rectangle::new(70.0, 40.0);

        let reconstructed =
            Capsule2d::try_from_shape(&shape).expect("Failed to reconstruct wide capsule");

        // With bounds (70, 40):
        // - radius = min(70, 40) = 40
        // - half_length_from_bounds = 40 - 40 = 0
        // - Capsule2d::new(40, 0*2) creates a capsule with radius=20, half_length=0
        // (because Capsule2d::new divides both parameters by 2 internally)
        assert_eq!(
            reconstructed.radius, 20.0,
            "Radius should match shader: min(bounds.x, bounds.y) / 2"
        );
        assert_eq!(
            reconstructed.half_length, 0.0,
            "Half_length should be 0 for wide bounds"
        );

        // Ensure it's not negative
        assert!(
            reconstructed.half_length >= 0.0,
            "Half_length must not be negative"
        );
    }

    #[test]
    fn test_circle_round_trip() {
        let original = Circle::new(25.0);
        let shape = SmudShape::from(original);
        let reconstructed = Circle::try_from_shape(&shape).expect("Failed to reconstruct circle");

        assert_eq!(
            original.radius, reconstructed.radius,
            "Circle radius should match after round-trip conversion"
        );
    }

    #[test]
    fn test_ellipse_round_trip() {
        let original = Ellipse::new(40.0, 25.0);
        let shape = SmudShape::from(original);
        let reconstructed = Ellipse::try_from_shape(&shape).expect("Failed to reconstruct ellipse");

        assert_eq!(
            original.half_size, reconstructed.half_size,
            "Ellipse half_size should match after round-trip conversion"
        );
    }

    #[test]
    fn test_annulus_round_trip() {
        let original = Annulus::new(15.0, 30.0);
        let shape = SmudShape::from(original);
        let reconstructed = Annulus::try_from_shape(&shape).expect("Failed to reconstruct annulus");

        assert_eq!(
            original.inner_circle.radius, reconstructed.inner_circle.radius,
            "Annulus inner_radius should match after round-trip conversion"
        );
        assert_eq!(
            original.outer_circle.radius, reconstructed.outer_circle.radius,
            "Annulus outer_radius should match after round-trip conversion"
        );
    }

    #[test]
    fn test_rhombus_round_trip() {
        let original = Rhombus::new(30.0, 40.0);
        let shape = SmudShape::from(original);
        let reconstructed = Rhombus::try_from_shape(&shape).expect("Failed to reconstruct rhombus");

        assert_eq!(
            original.half_diagonals, reconstructed.half_diagonals,
            "Rhombus half_diagonals should match after round-trip conversion"
        );
    }

    #[test]
    fn test_circular_sector_round_trip() {
        let original = CircularSector::from_turns(35.0, 0.25);
        let shape = SmudShape::from(original);
        let reconstructed =
            CircularSector::try_from_shape(&shape).expect("Failed to reconstruct circular sector");

        assert_eq!(
            original.arc.radius, reconstructed.arc.radius,
            "CircularSector radius should match after round-trip conversion"
        );
        assert_eq!(
            original.arc.half_angle, reconstructed.arc.half_angle,
            "CircularSector half_angle should match after round-trip conversion"
        );
    }

    #[test]
    fn test_regular_polygon_round_trip() {
        let original = RegularPolygon::new(30.0, 6);
        let shape = SmudShape::from(original);
        let reconstructed =
            RegularPolygon::try_from_shape(&shape).expect("Failed to reconstruct regular polygon");

        assert_eq!(
            original.circumcircle.radius, reconstructed.circumcircle.radius,
            "RegularPolygon radius should match after round-trip conversion"
        );
        assert_eq!(
            original.sides, reconstructed.sides,
            "RegularPolygon sides should match after round-trip conversion"
        );
    }
}
