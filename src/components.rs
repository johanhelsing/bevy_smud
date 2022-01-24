use bevy::{ecs::query::QueryItem, prelude::*, render::render_component::ExtractComponent};

#[derive(Component, Clone)]
pub struct SmudShape {
    pub color: Color,
}

impl Default for SmudShape {
    fn default() -> Self {
        Self { color: Color::PINK }
    }
}

impl ExtractComponent for SmudShape {
    type Query = &'static SmudShape;
    type Filter = ();

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}
