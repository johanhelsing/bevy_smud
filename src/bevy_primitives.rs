//! Convenience conversions from Bevy's math primitives to bevy_smud shapes.
//!
//! This module provides `From` trait implementations that allow you to create
//! `SmudShape` components directly from Bevy's 2D primitive shapes like `Rectangle`,
//! `Circle`, etc. When the `bevy_picking` feature is enabled, it also automatically
//! adds precise picking support via an observer.

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::color::palettes::css;
use bevy::math::primitives::{Circle, Rectangle};
use bevy::prelude::*;

use crate::{SmudShape, DEFAULT_FILL_HANDLE};

#[cfg(feature = "bevy_picking")]
use crate::sdf;

/// Parametrized rectangle shape SDF
pub const RECTANGLE_SDF_HANDLE: Handle<Shader> = uuid_handle!("2289ee84-18da-4e35-87b2-e256fd88c092");

/// Parametrized circle shape SDF
pub const CIRCLE_SDF_HANDLE: Handle<Shader> = uuid_handle!("abb54e5e-62f3-4ea2-9604-84368bb6ae6d");

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

        // Register observers for auto-adding picking shapes
        #[cfg(feature = "bevy_picking")]
        app.add_observer(auto_add_picking_shape);
    }
}

// ============================================================================
// SmudShape conversions
// ============================================================================

impl From<Rectangle> for SmudShape {
    /// Create a SmudShape from a Bevy Rectangle primitive.
    ///
    /// This creates a parametrized rectangle SDF with the rectangle's half-size
    /// stored in params.xy. The bounds are automatically set with some padding
    /// to prevent clipping.
    ///
    /// When the `bevy_picking` feature is enabled, a `SmudPickingShape` component
    /// will be automatically added to the entity for precise hit-testing. This happens
    /// via an observer, so you don't need to manually add the picking component!
    ///
    /// # Example
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_smud::prelude::*;
    /// # let mut commands: Commands = panic!();
    /// // SmudPickingShape is automatically added when bevy_picking is enabled!
    /// commands.spawn((
    ///     Transform::from_translation(Vec3::new(100., 0., 0.)),
    ///     SmudShape::from(Rectangle::new(100., 50.))
    ///         .with_color(Color::srgb(0.8, 0.2, 0.2))
    /// ));
    /// ```
    fn from(rect: Rectangle) -> Self {
        // Use the pre-loaded rectangle SDF shader
        let sdf = RECTANGLE_SDF_HANDLE;

        // Rectangle bounds with padding (2px on each side for anti-aliasing)
        let padding = 2.0;
        let bounds = Rectangle {
            half_size: rect.half_size + Vec2::splat(padding),
        };

        // Store the half-size in params.xy for the shader
        let params = Vec4::new(rect.half_size.x, rect.half_size.y, 0.0, 0.0);

        Self {
            color: css::WHITE.into(),
            sdf,
            fill: DEFAULT_FILL_HANDLE,
            bounds,
            params,
            blend_mode: Default::default(),
        }
    }
}

impl From<Circle> for SmudShape {
    /// Create a SmudShape from a Bevy Circle primitive.
    ///
    /// This creates a parametrized circle SDF with the circle's radius
    /// stored in params.x. The bounds are automatically set with some padding
    /// to prevent clipping.
    ///
    /// When the `bevy_picking` feature is enabled, a `SmudPickingShape` component
    /// will be automatically added to the entity for precise hit-testing.
    ///
    /// # Example
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_smud::prelude::*;
    /// # let mut commands: Commands = panic!();
    /// commands.spawn((
    ///     Transform::from_translation(Vec3::new(100., 0., 0.)),
    ///     SmudShape::from(Circle::new(50.))
    ///         .with_color(Color::srgb(0.2, 0.8, 0.2))
    /// ));
    /// ```
    fn from(circle: Circle) -> Self {
        // Use the pre-loaded circle SDF shader
        let sdf = CIRCLE_SDF_HANDLE;

        // Circle bounds with padding (2px on each side for anti-aliasing)
        let padding = 2.0;
        let size = circle.radius * 2.0 + padding * 2.0;
        let bounds = Rectangle::new(size, size);

        // Store the radius in params.x for the shader
        let params = Vec4::new(circle.radius, 0.0, 0.0, 0.0);

        Self {
            color: css::WHITE.into(),
            sdf,
            fill: DEFAULT_FILL_HANDLE,
            bounds,
            params,
            blend_mode: Default::default(),
        }
    }
}

// ============================================================================
// SmudPickingShape conversions (when bevy_picking feature is enabled)
// ============================================================================

#[cfg(feature = "bevy_picking")]
impl From<Rectangle> for crate::picking_backend::SmudPickingShape {
    /// Create a `SmudPickingShape` from a Bevy `Rectangle` primitive.
    ///
    /// This provides precise hit-testing for rectangle shapes by using the CPU-side
    /// SDF function. The picking shape will exactly match the visual shape created
    /// by `SmudShape::from(Rectangle)`.
    ///
    /// Note: When using `SmudShape::from(Rectangle)`, the picking shape is automatically
    /// added via an observer. You only need to use this manually if you're not using
    /// the `From` trait or want explicit control.
    ///
    /// # Example
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_smud::prelude::*;
    /// # let mut commands: Commands = panic!();
    /// let rect = Rectangle::new(100., 50.);
    /// commands.spawn((
    ///     Transform::from_translation(Vec3::new(100., 0., 0.)),
    ///     SmudShape::from(rect).with_color(Color::srgb(0.8, 0.2, 0.2)),
    ///     SmudPickingShape::from(rect), // Explicit picking (usually not needed)
    /// ));
    /// ```
    fn from(rect: Rectangle) -> Self {
        let half_size = rect.half_size;
        Self::new(move |p| sdf::sd_box(p, half_size))
    }
}

#[cfg(feature = "bevy_picking")]
impl From<Circle> for crate::picking_backend::SmudPickingShape {
    /// Create a `SmudPickingShape` from a Bevy `Circle` primitive.
    ///
    /// This provides precise hit-testing for circle shapes by using the CPU-side
    /// SDF function. The picking shape will exactly match the visual shape created
    /// by `SmudShape::from(Circle)`.
    ///
    /// Note: When using `SmudShape::from(Circle)`, the picking shape is automatically
    /// added via an observer.
    ///
    /// # Example
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_smud::prelude::*;
    /// # let mut commands: Commands = panic!();
    /// let circle = Circle::new(50.);
    /// commands.spawn((
    ///     Transform::from_translation(Vec3::new(100., 0., 0.)),
    ///     SmudShape::from(circle).with_color(Color::srgb(0.2, 0.8, 0.2)),
    ///     SmudPickingShape::from(circle), // Explicit picking (usually not needed)
    /// ));
    /// ```
    fn from(circle: Circle) -> Self {
        let radius = circle.radius;
        Self::new(move |p| sdf::circle(p, radius))
    }
}

// ============================================================================
// Observer for auto-adding picking shapes
// ============================================================================

/// Observer that automatically adds SmudPickingShape for shapes created from primitives
#[cfg(feature = "bevy_picking")]
fn auto_add_picking_shape(
    trigger: On<Add, SmudShape>,
    query: Query<&SmudShape>,
    mut commands: Commands,
) {
    use crate::picking_backend::SmudPickingShape;

    let entity = trigger.entity;
    if let Ok(shape) = query.get(entity) {
        let picking_shape = if shape.sdf.id() == RECTANGLE_SDF_HANDLE.id() {
            // Rectangle: Extract half-size from params.xy
            let half_size = Vec2::new(shape.params.x, shape.params.y);
            Some(SmudPickingShape::new(move |p| sdf::sd_box(p, half_size)))
        } else if shape.sdf.id() == CIRCLE_SDF_HANDLE.id() {
            // Circle: Extract radius from params.x
            let radius = shape.params.x;
            Some(SmudPickingShape::new(move |p| sdf::circle(p, radius)))
        } else {
            None
        };

        if let Some(picking_shape) = picking_shape {
            commands.entity(entity).insert(picking_shape);
        }
    }
}
