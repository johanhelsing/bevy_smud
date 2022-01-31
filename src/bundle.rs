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

#[derive(Bundle, Default, Clone)]
pub struct UiShapeBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    // /// Describes the color of the node
    // pub color: UiColor,
    pub shape: SmudShape,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Describes the color of the node, will be multiplied with the shape color
    pub color: UiColor,
}
