use bevy::{ecs::query::QueryItem, prelude::*, render::render_component::ExtractComponent};

#[derive(Component, Clone)]
pub enum SmudShape {
    Arc(f32),
}

impl Default for SmudShape {
    fn default() -> Self {
        SmudShape::Arc(1.)
    }
}

impl ExtractComponent for SmudShape {
    type Query = &'static SmudShape;
    type Filter = ();

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}
