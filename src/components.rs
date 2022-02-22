use bevy::{ecs::query::QueryItem, prelude::*, render::render_component::ExtractComponent};

use crate::DEFAULT_FILL_HANDLE;

#[derive(Component, Debug, Clone)]
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
}

impl Default for SmudShape {
    fn default() -> Self {
        Self {
            color: Color::PINK,
            sdf: Default::default(),
            frame: Default::default(),
            fill: DEFAULT_FILL_HANDLE.typed(),
        }
    }
}

impl ExtractComponent for SmudShape {
    type Query = &'static SmudShape;
    type Filter = ();

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

/// Bounds for describing how far the fragment shader of a shape will reach, should be bigger than the shape unless you want to clip it
#[derive(Debug, Clone, Copy)]
pub enum Frame {
    /// A quad with a given half-size (!)
    Quad(f32), // todo: it probably makes sense for this to be the full width instead...
}

impl Frame {
    const DEFAULT_QUAD: Self = Self::Quad(1.);
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT_QUAD
    }
}
