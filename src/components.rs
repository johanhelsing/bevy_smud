use bevy::{ecs::query::QueryItem, prelude::*, render::render_component::ExtractComponent};

#[derive(Component, Clone)]
pub struct SmudShape {
    pub color: Color,
    pub sdf_shader: Handle<Shader>,
}

impl Default for SmudShape {
    fn default() -> Self {
        Self {
            color: Color::PINK,
            sdf_shader: Default::default(), // todo
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
