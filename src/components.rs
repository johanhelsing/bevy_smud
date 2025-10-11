use bevy::camera::visibility::{VisibilityClass, add_visibility_class};
use bevy::color::palettes::css;
use bevy::math::primitives::Rectangle;
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
    /// The bounds for rendering this shape, should be larger than the actual SDF shape to avoid clipping
    pub bounds: Rectangle,
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
            bounds: default(),
            params: default(),
            fill: DEFAULT_FILL_HANDLE,
            blend_mode: BlendMode::default(),
        }
    }
}

impl SmudShape {
    /// Set the color for this shape (builder pattern)
    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the fill shader for this shape (builder pattern)
    pub fn with_fill(mut self, fill: Handle<Shader>) -> Self {
        self.fill = fill;
        self
    }

    /// Set the blend mode for this shape (builder pattern)
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}
