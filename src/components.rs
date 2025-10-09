use bevy::camera::visibility::{VisibilityClass, add_visibility_class};
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::render::sync_world::SyncToRenderWorld;

use crate::DEFAULT_FILL_HANDLE;

/// Blend mode for shapes
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// Standard alpha blending
    #[default]
    Alpha,
    /// Additive blending (colors are added together)
    Additive,
}

#[derive(Component, Reflect, Debug, Clone)]
#[require(Transform, Visibility, SyncToRenderWorld, VisibilityClass)] // TODO: anchor?
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = add_visibility_class::<SmudShape>)]
/// Main component used for describing an sdf shape
pub struct SmudShape {
    /// The color used by the fill shader
    pub color: Color,
    /// Shader containing a wgsl function for a signed distance field
    ///
    /// The shader needs to have the signature `fn sdf(p: vec2<f32>) -> f32`.
    pub sdf: Handle<Shader>,
    /// Shader containing a wgsl function for the fill of the shape
    ///
    /// The shader needs to have the signature `fn fill(distance: f32, color: vec4<f32>) -> vec4<f32>`.
    pub fill: Handle<Shader>, // todo: wrap in newtypes?
    /// The outer bounds for the shape, should be bigger than the sdf shape
    pub frame: Frame,
    /// Parameters to pass to shapes, for things such as width of a box
    // perhaps it would be a better idea to have this as a separate component?
    // keeping it here for now...
    pub params: Vec4,
    /// Blend mode for the shape
    pub blend_mode: BlendMode,
}

impl Default for SmudShape {
    fn default() -> Self {
        Self {
            color: css::PINK.into(),
            sdf: default(),
            frame: default(),
            params: default(),
            fill: DEFAULT_FILL_HANDLE,
            blend_mode: BlendMode::default(),
        }
    }
}

/// Bounds for describing how far the fragment shader of a shape will reach, should be bigger than the shape unless you want to clip it
#[derive(Reflect, Debug, Clone, Copy)]
pub enum Frame {
    /// A quad with a given half-size
    Quad {
        /// The half-size of the quad (distance from center to edge)
        half_size: f32,
    },
}

impl Frame {
    const DEFAULT_QUAD: Self = Self::Quad { half_size: 1. };

    /// Create a quad frame from a half-size (distance from center to edge)
    pub const fn quad_half_size(half_size: f32) -> Self {
        Self::Quad { half_size }
    }

    /// Create a quad frame from a full size (total width/height)
    pub const fn quad_size(size: f32) -> Self {
        Self::Quad {
            half_size: size / 2.0,
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT_QUAD
    }
}
