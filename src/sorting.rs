//! Draws shapes at the same depth grouped by pipeline, so they batch.
//!
//! Bevy sorts the 2D transparent phase by z alone, and keeps shapes at equal z
//! in the order they were queued. A scene that mixes many sdf and fill pairs
//! at one z then switches pipeline on almost every shape, one draw call each.

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_phase::{ViewSortedRenderPhases, sort_phase_system},
    },
};

pub(crate) struct ShapeSortingPlugin;

impl Plugin for ShapeSortingPlugin {
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                sort_by_pipeline
                    .in_set(RenderSystems::PhaseSort)
                    .after(sort_phase_system::<Transparent2d>),
            );
        }
    }
}

fn sort_by_pipeline(mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>) {
    for phase in phases.values_mut() {
        phase.items.sort_by(|_, a, _, b| {
            a.sort_key
                .cmp(&b.sort_key)
                .then_with(|| a.pipeline.id().cmp(&b.pipeline.id()))
        });
    }
}
