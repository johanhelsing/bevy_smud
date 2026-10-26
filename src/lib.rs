#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

pub use components::*;
pub use shader_loading::{DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE};

#[cfg(not(feature = "webgl2"))]
use render::ShapeRenderPlugin;
#[cfg(feature = "webgl2")]
use render_webgl2::ShapeRenderPlugin;
use shader_loading::ShaderLoadingPlugin;
#[cfg(feature = "bevy_ui")]
use ui::UiShapePlugin;

#[cfg(feature = "bevy_primitives")]
pub mod bevy_primitives;
mod components;
#[cfg(feature = "bevy_picking")]
mod picking_backend;
#[cfg(not(feature = "webgl2"))]
mod render;
#[cfg(feature = "webgl2")]
mod render_webgl2;
pub mod sdf;
mod sdf_assets;
mod shader_loading;
#[cfg(feature = "bevy_ui")]
mod ui;
mod util;

#[cfg(not(feature = "webgl2"))]
pub use render::{SmudMaterial, SmudMaterialKey};

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_smud::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        BlendMode, DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE, SmudPlugin, SmudShape,
        sdf_assets::SdfAssets,
    };

    #[cfg(feature = "bevy_primitives")]
    pub use bevy::shape::{Annulus, Circle, Ellipse, Rectangle};

    #[cfg(feature = "bevy_picking")]
    pub use crate::picking_backend::{
        SmudPickingCamera, SmudPickingPlugin, SmudPickingSettings, SmudPickingShape,
    };

    #[cfg(feature = "bevy_ui")]
    pub use crate::ui::UiShape;

    pub use crate::sdf;
}

#[derive(Default)]
/// Main plugin for enabling rendering of Sdf shapes
pub struct SmudPlugin;

/// System set for shape rendering.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ShapeRenderSystems {
    /// Prepares new and changed shapes for rendering: in `PostUpdate`, or in
    /// `ExtractSchedule` with the `webgl2` feature.
    PrepareShapes,
}

impl Plugin for SmudPlugin {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugins((ShaderLoadingPlugin, ShapeRenderPlugin));

        #[cfg(feature = "bevy_primitives")]
        app.add_plugins(bevy_primitives::BevyPrimitivesPlugin);

        #[cfg(feature = "bevy_ui")]
        app.add_plugins(UiShapePlugin);

        app.register_type::<SmudShape>();
    }
}
