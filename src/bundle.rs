use bevy::prelude::*;

use crate::SmudShape;

#[derive(Bundle, Default, Clone, Debug)]
/// Bundle with all the components needed for drawing an sdf shape in 2d world space
pub struct ShapeBundle {
    pub shape: SmudShape,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

#[derive(Bundle, Default, Clone, Debug)]
/// Bundle with all the components used for drawing an sdf shape as a bevy UI node
pub struct UiShapeBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Describes the actual shape and its fill
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
