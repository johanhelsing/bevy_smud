//! Convenience conversions from Bevy's math primitives to bevy_smud shapes.
//!
//! This module provides `From` trait implementations that allow you to create
//! `SmudShape` components directly from Bevy's 2D primitive shapes like [`Rectangle`],
//! [`Circle`], and [`Ellipse`].
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
//! ```
//!
//! # Picking Support
//!
//! When the `bevy_picking` feature is enabled, `SmudPickingShape` is automatically
//! added to entities with primitive-based shapes for precise hit-testing.

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::color::palettes::css;
use bevy::math::primitives::{Circle, Ellipse, Rectangle};
use bevy::prelude::*;

use crate::{SmudShape, DEFAULT_FILL_HANDLE};

#[cfg(feature = "bevy_picking")]
use crate::sdf;

/// Parametrized rectangle shape SDF
pub const RECTANGLE_SDF_HANDLE: Handle<Shader> = uuid_handle!("2289ee84-18da-4e35-87b2-e256fd88c092");

/// Parametrized circle shape SDF
pub const CIRCLE_SDF_HANDLE: Handle<Shader> = uuid_handle!("abb54e5e-62f3-4ea2-9604-84368bb6ae6d");

/// Parametrized ellipse shape SDF
pub const ELLIPSE_SDF_HANDLE: Handle<Shader> = uuid_handle!("2c02adad-84fb-46d7-8ef8-f4b6d86d6149");

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

        // Register observers for auto-adding picking shapes
        #[cfg(feature = "bevy_picking")]
        app.add_observer(auto_add_picking_shape);
    }
}

impl From<Rectangle> for SmudShape {
    /// Creates a [`SmudShape`] from a Bevy [`Rectangle`] primitive.
    ///
    /// See the [module documentation](self) for more information and examples.
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
    /// Creates a [`SmudShape`] from a Bevy [`Circle`] primitive.
    ///
    /// See the [module documentation](self) for more information and examples.
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

impl From<Ellipse> for SmudShape {
    /// Creates a [`SmudShape`] from a Bevy [`Ellipse`] primitive.
    ///
    /// Note: If the ellipse has equal radii (i.e., it's actually a circle),
    /// the circle SDF is used instead as sd_ellipse doesn't handle that case.
    ///
    /// See the [module documentation](self) for more information and examples.
    fn from(ellipse: Ellipse) -> Self {
        let padding = 2.0;

        // Check if the ellipse is actually a circle (equal radii)
        // Use a small epsilon for floating point comparison
        const EPSILON: f32 = 1e-6;
        if (ellipse.half_size.x - ellipse.half_size.y).abs() < EPSILON {
            // It's a circle, use the circle SDF
            let sdf = CIRCLE_SDF_HANDLE;
            let radius = ellipse.half_size.x;
            let size = radius * 2.0 + padding * 2.0;
            let bounds = Rectangle::new(size, size);
            let params = Vec4::new(radius, 0.0, 0.0, 0.0);

            Self {
                color: css::WHITE.into(),
                sdf,
                fill: DEFAULT_FILL_HANDLE,
                bounds,
                params,
                blend_mode: Default::default(),
            }
        } else {
            // It's a proper ellipse, use the ellipse SDF
            let sdf = ELLIPSE_SDF_HANDLE;
            let bounds = Rectangle {
                half_size: ellipse.half_size + Vec2::splat(padding),
            };
            let params = Vec4::new(ellipse.half_size.x, ellipse.half_size.y, 0.0, 0.0);

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
}

#[cfg(feature = "bevy_picking")]
impl From<Rectangle> for crate::picking_backend::SmudPickingShape {
    /// Creates a [`SmudPickingShape`](crate::picking_backend::SmudPickingShape) from a Bevy [`Rectangle`] primitive.
    ///
    /// Note: This is typically not needed as picking shapes are added automatically.
    /// See the [module documentation](self) for more information.
    fn from(rect: Rectangle) -> Self {
        let half_size = rect.half_size;
        Self::new(move |p| sdf::sd_box(p, half_size))
    }
}

#[cfg(feature = "bevy_picking")]
impl From<Circle> for crate::picking_backend::SmudPickingShape {
    /// Creates a [`SmudPickingShape`](crate::picking_backend::SmudPickingShape) from a Bevy [`Circle`] primitive.
    ///
    /// Note: This is typically not needed as picking shapes are added automatically.
    /// See the [module documentation](self) for more information.
    fn from(circle: Circle) -> Self {
        let radius = circle.radius;
        Self::new(move |p| sdf::circle(p, radius))
    }
}

#[cfg(feature = "bevy_picking")]
impl From<Ellipse> for crate::picking_backend::SmudPickingShape {
    /// Creates a [`SmudPickingShape`](crate::picking_backend::SmudPickingShape) from a Bevy [`Ellipse`] primitive.
    ///
    /// Note: This is typically not needed as picking shapes are added automatically.
    /// See the [module documentation](self) for more information.
    fn from(ellipse: Ellipse) -> Self {
        // Check if it's actually a circle
        const EPSILON: f32 = 1e-6;
        if (ellipse.half_size.x - ellipse.half_size.y).abs() < EPSILON {
            // Use circle SDF for equal radii
            let radius = ellipse.half_size.x;
            Self::new(move |p| sdf::circle(p, radius))
        } else {
            // Use ellipse SDF
            let a = ellipse.half_size.x;
            let b = ellipse.half_size.y;
            Self::new(move |p| sdf::ellipse(p, a, b))
        }
    }
}

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
            // Note: This handles both Circle primitives and Ellipse primitives with equal radii
            let radius = shape.params.x;
            Some(SmudPickingShape::new(move |p| sdf::circle(p, radius)))
        } else if shape.sdf.id() == ELLIPSE_SDF_HANDLE.id() {
            // Ellipse: Extract semi-axes from params.xy
            let a = shape.params.x;
            let b = shape.params.y;
            Some(SmudPickingShape::new(move |p| sdf::ellipse(p, a, b)))
        } else {
            None
        };

        if let Some(picking_shape) = picking_shape {
            commands.entity(entity).insert(picking_shape);
        }
    }
}
