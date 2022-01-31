use bevy::prelude::*;

use crate::SmudShape;

#[derive(Bundle, Default, Clone)]
pub struct ShapeBundle {
    pub shape: SmudShape,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

impl Default for ShapeBundle {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
