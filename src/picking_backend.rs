//! A picking backend for bevy_smud SDF shapes.
//!
//! This backend allows clicking and hovering over SDF shapes rendered by bevy_smud.
//! It uses a simple frame-based approach where shapes are considered "hit" if the pointer
//! is within their frame bounds.
//!
//! ## Usage
//!
//! This backend does not require markers on cameras or entities to function by default.
//! However, you can enable `SmudPickingSettings::require_markers` to make picking opt-in
//! by adding `SmudPickingCamera` to cameras and `Pickable` to entities.
//!
//! ## Implementation Notes
//!
//! - The backend considers a shape picked if the pointer is within the shape's frame bounds
//! - The `position` reported in `HitData` is in world space
//! - The `normal` points away from the shape using the transform's back vector
//! - Depth is calculated based on the shape's Z position in camera space

use bevy::{picking::PickSet, picking::backend::prelude::*, prelude::*};

use crate::{Frame, SmudShape};

/// An optional component that marks cameras that should be used for SDF shape picking.
///
/// Only needed if [`SmudPickingSettings::require_markers`] is set to `true`, and ignored
/// otherwise.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct SmudPickingCamera;

/// Runtime settings for SDF shape picking.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct SmudPickingSettings {
    /// When set to `true`, SDF shape picking will only consider cameras marked with
    /// [`SmudPickingCamera`] and entities marked with [`Pickable`]. `false` by default.
    ///
    /// This setting provides fine-grained control over which cameras and entities
    /// should be used by the SDF shape picking backend at runtime.
    pub require_markers: bool,
}

/// A plugin that adds picking support for SDF shapes rendered by bevy_smud.
#[derive(Clone)]
pub struct SmudPickingPlugin;

impl Plugin for SmudPickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SmudPickingSettings>()
            .add_systems(PreUpdate, smud_picking.in_set(PickSet::Backend));
    }
}

/// The main picking system that tests pointer intersections with SDF shapes.
pub fn smud_picking(
    ray_map: Res<RayMap>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, Has<SmudPickingCamera>)>,
    settings: Res<SmudPickingSettings>,
    shapes: Query<(
        Entity,
        &SmudShape,
        &GlobalTransform,
        &ViewVisibility,
        Option<&Pickable>,
    )>,
    mut output: EventWriter<PointerHits>,
) {
    // Collect shapes sorted by depth (back to front for proper ordering)
    let mut sorted_shapes: Vec<_> = shapes
        .iter()
        .filter_map(|(entity, shape, transform, visibility, pickable)| {
            // Skip if visibility is off or transform is invalid
            if !visibility.get() || transform.affine().is_nan() {
                return None;
            }

            // If markers are required, check if entity has Pickable component
            if settings.require_markers && pickable.is_none() {
                return None;
            }

            // If entity has Pickable component, check if it's hoverable
            if let Some(pickable) = pickable
                && !pickable.is_hoverable
            {
                return None;
            }

            Some((entity, shape, transform, pickable))
        })
        .collect();

    // Sort by Z coordinate (back to front)
    sorted_shapes.sort_by(|(_, _, transform_a, _), (_, _, transform_b, _)| {
        transform_b
            .translation()
            .z
            .total_cmp(&transform_a.translation().z)
    });

    for (&ray_id, &ray) in ray_map.map.iter() {
        let (camera_entity, pointer) = (ray_id.camera, ray_id.pointer);
        // Check if this camera should be considered for picking
        let Ok((cam_entity, camera, cam_transform, cam_can_pick)) = cameras.get(camera_entity)
        else {
            continue;
        };

        let marker_requirement = !settings.require_markers || cam_can_pick;
        if !camera.is_active || !marker_requirement {
            continue;
        }

        let mut picks = Vec::new();
        let mut blocked = false;

        // Test intersection with each shape
        for (entity, shape, shape_transform, pickable) in &sorted_shapes {
            if blocked {
                break;
            }

            // Project the ray onto the shape's plane (assuming Z plane for 2D shapes)
            let shape_z = shape_transform.translation().z;

            // Calculate where the ray intersects the shape's Z plane
            let ray_start = ray.origin;
            let ray_direction = ray.direction;

            // If ray is parallel to the Z plane, skip this shape
            if ray_direction.z.abs() < f32::EPSILON {
                continue;
            }

            let t = (shape_z - ray_start.z) / ray_direction.z;
            let intersection_point = ray_start + ray_direction * t;

            // Transform the intersection point to shape local space
            let world_to_shape = shape_transform.affine().inverse();
            let local_point = world_to_shape.transform_point3(intersection_point);

            // Check if the point is within the shape's frame
            let is_within_frame = match shape.frame {
                Frame::Quad(half_size) => {
                    local_point.x.abs() <= half_size && local_point.y.abs() <= half_size
                }
            };

            if is_within_frame {
                // Calculate depth from camera's near plane
                let hit_pos_cam = cam_transform
                    .affine()
                    .inverse()
                    .transform_point3(intersection_point);
                let depth = -hit_pos_cam.z;

                picks.push((
                    *entity,
                    HitData::new(
                        cam_entity,
                        depth,
                        Some(intersection_point),
                        Some(*shape_transform.back()),
                    ),
                ));

                // Check if this shape should block shapes behind it
                if let Some(pickable) = pickable {
                    blocked = pickable.should_block_lower;
                } else {
                    // Default behavior: block shapes behind
                    blocked = true;
                }
            }
        }

        if !picks.is_empty() {
            let order = camera.order as f32;
            output.write(PointerHits::new(pointer, picks, order));
        }
    }
}
