use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

use crate::SmudShape;

pub(crate) struct InspectablePlugin;

impl Plugin for InspectablePlugin {
    fn build(&self, app: &mut App) {
        let mut inspectable_registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        inspectable_registry.register::<SmudShape>();

        // NOTE: while this seems cleaner, it panics if bevy_smud is loaded before
        // the bevy-inspector-egui plugin.
        // inspectable_registry.register_inspectable::<SmudShape>();
    }
}
