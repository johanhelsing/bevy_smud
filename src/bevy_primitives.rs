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
use bevy::color::palettes::css;
use bevy::math::bounding::{Bounded2d, BoundingVolume};
use bevy::math::primitives::{Annulus, Circle, Ellipse, Rectangle};
use bevy::prelude::*;

use crate::{DEFAULT_FILL_HANDLE, SmudShape};

#[cfg(feature = "bevy_picking")]
use crate::sdf;

/// Trait for primitives that can be converted to SmudShape.
///
/// This trait encapsulates the varying parts of primitive-to-shape conversion:
/// shader handles, parameter extraction, bounds calculation, and picking functions.
trait SmudPrimitive: Sized + Bounded2d {
    /// The shader handle for this primitive's SDF
    fn sdf_shader() -> Handle<Shader>;

    /// Extract bounds for rendering (including padding for anti-aliasing)
    ///
    /// Default implementation uses the `Bounded2d` trait to compute an AABB
    /// and adds padding for anti-aliasing.
    fn bounds(&self) -> Rectangle {
        const PADDING: f32 = 2.0;
        let aabb = Bounded2d::aabb_2d(self, Vec2::ZERO);
        Rectangle {
            half_size: aabb.half_size() + Vec2::splat(PADDING),
        }
    }

    /// Extract shader parameters (stored in SmudShape.params)
    fn params(&self) -> Vec4;

    /// Try to reconstruct a primitive from a SmudShape's shader handle and params
    fn try_from_shape(sdf: &Handle<Shader>, params: Vec4) -> Option<Self>;

    #[cfg(feature = "bevy_picking")]
    /// Create a picking closure for this primitive
    fn picking_fn(&self) -> Box<dyn Fn(Vec2) -> f32 + Send + Sync>;
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

        // Register observers for auto-adding picking shapes
        #[cfg(feature = "bevy_picking")]
        app.add_observer(auto_add_picking_shape);
    }
}

impl SmudPrimitive for Rectangle {
    fn sdf_shader() -> Handle<Shader> {
        RECTANGLE_SDF_HANDLE
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.half_size.x, self.half_size.y, 0.0, 0.0)
    }

    fn try_from_shape(sdf: &Handle<Shader>, params: Vec4) -> Option<Self> {
        if sdf.id() == RECTANGLE_SDF_HANDLE.id() {
            Some(Rectangle {
                half_size: Vec2::new(params.x, params.y),
            })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(Vec2) -> f32 + Send + Sync> {
        let half_size = self.half_size;
        Box::new(move |p| sdf::sd_box(p, half_size))
    }
}

impl SmudPrimitive for Circle {
    fn sdf_shader() -> Handle<Shader> {
        CIRCLE_SDF_HANDLE
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.radius, 0.0, 0.0, 0.0)
    }

    fn try_from_shape(sdf: &Handle<Shader>, params: Vec4) -> Option<Self> {
        if sdf.id() == CIRCLE_SDF_HANDLE.id() {
            Some(Circle { radius: params.x })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(Vec2) -> f32 + Send + Sync> {
        let radius = self.radius;
        Box::new(move |p| sdf::circle(p, radius))
    }
}

impl SmudPrimitive for Ellipse {
    fn sdf_shader() -> Handle<Shader> {
        ELLIPSE_SDF_HANDLE
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.half_size.x, self.half_size.y, 0.0, 0.0)
    }

    fn try_from_shape(sdf: &Handle<Shader>, params: Vec4) -> Option<Self> {
        if sdf.id() == ELLIPSE_SDF_HANDLE.id() {
            Some(Ellipse {
                half_size: Vec2::new(params.x, params.y),
            })
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(Vec2) -> f32 + Send + Sync> {
        let a = self.half_size.x;
        let b = self.half_size.y;
        Box::new(move |p| {
            const EPSILON: f32 = 1e-6;
            if (a - b).abs() < EPSILON {
                sdf::circle(p, a)
            } else {
                sdf::ellipse(p, a, b)
            }
        })
    }
}

impl SmudPrimitive for Annulus {
    fn sdf_shader() -> Handle<Shader> {
        ANNULUS_SDF_HANDLE
    }

    fn params(&self) -> Vec4 {
        Vec4::new(self.outer_circle.radius, self.inner_circle.radius, 0.0, 0.0)
    }

    fn try_from_shape(sdf: &Handle<Shader>, params: Vec4) -> Option<Self> {
        if sdf.id() == ANNULUS_SDF_HANDLE.id() {
            Some(Annulus::new(params.y, params.x))
        } else {
            None
        }
    }

    #[cfg(feature = "bevy_picking")]
    fn picking_fn(&self) -> Box<dyn Fn(Vec2) -> f32 + Send + Sync> {
        let outer_radius = self.outer_circle.radius;
        let inner_radius = self.inner_circle.radius;
        Box::new(move |p| sdf::annulus(p, outer_radius, inner_radius))
    }
}

impl<T: SmudPrimitive> From<T> for SmudShape {
    fn from(primitive: T) -> Self {
        Self {
            color: css::WHITE.into(),
            sdf: T::sdf_shader(),
            fill: DEFAULT_FILL_HANDLE,
            bounds: primitive.bounds(),
            params: primitive.params(),
            blend_mode: Default::default(),
        }
    }
}

#[cfg(feature = "bevy_picking")]
impl<T: SmudPrimitive> From<T> for crate::picking_backend::SmudPickingShape {
    fn from(primitive: T) -> Self {
        Self::new(primitive.picking_fn())
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
        // Try to reconstruct the primitive and use its picking function
        let picking_shape = Rectangle::try_from_shape(&shape.sdf, shape.params)
            .map(|p| SmudPickingShape::from(p))
            .or_else(|| Circle::try_from_shape(&shape.sdf, shape.params)
                .map(|p| SmudPickingShape::from(p)))
            .or_else(|| Ellipse::try_from_shape(&shape.sdf, shape.params)
                .map(|p| SmudPickingShape::from(p)))
            .or_else(|| Annulus::try_from_shape(&shape.sdf, shape.params)
                .map(|p| SmudPickingShape::from(p)));

        if let Some(picking_shape) = picking_shape {
            commands.entity(entity).insert(picking_shape);
        }
    }
}
