use bevy::{ecs::query::QueryItem, prelude::*, render::render_component::ExtractComponent};

use crate::DEFAULT_FILL_HANDLE;

#[derive(Component, Clone)]
pub struct SmudShape {
    pub color: Color,
    pub sdf_shader: Handle<Shader>,
    pub fill_shader: Handle<Shader>, // todo: wrap in newtypes?
    pub frame: Frame,
}

impl Default for SmudShape {
    fn default() -> Self {
        Self {
            color: Color::PINK,
            sdf_shader: Default::default(),
            frame: Default::default(),
            fill_shader: DEFAULT_FILL_HANDLE.typed(),
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

#[derive(Debug, Clone, Copy)]
pub enum Frame {
    Quad(f32),
}

impl Frame {
    const DEFAULT_QUAD: Self = Self::Quad(1.);
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT_QUAD
    }
}
