//! A picking backend for bevy_smud SDF shapes.
//!
//! This backend allows clicking and hovering over SDF shapes rendered by bevy_smud.
//! It uses a simple bounds-based approach where shapes are considered "hit" if the pointer
//! is within their bounding box.
//!
//! ## Usage
//!
//! This backend does not require markers on cameras or entities to function by default.
//! However, you can enable `SmudPickingSettings::require_markers` to make picking opt-in
//! by adding `SmudPickingCamera` to cameras and `Pickable` to entities.
//!
//! ## Implementation Notes
//!
//! - The backend considers a shape picked if the pointer is within the shape's bounding box
//! - The `position` reported in `HitData` is in world space
//! - The `normal` points away from the shape using the transform's back vector
//! - Depth is calculated based on the shape's Z position in camera space

use bevy::{picking::PickingSystems, picking::backend::prelude::*, prelude::*};

use crate::SmudShape;

/// Input parameters for SDF picking functions, matching the shader SdfInput struct.
#[derive(Debug, Clone, Copy)]
pub struct SdfInput {
    /// Position in local space
    pub pos: Vec2,
    /// Shape bounds (half-extents)
    pub bounds: Vec2,
    /// Shader parameters
    pub params: Vec4,
}

/// An optional component that marks cameras that should be used for SDF shape picking.
///
/// Only needed if [`SmudPickingSettings::require_markers`] is set to `true`, and ignored
/// otherwise.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Debug, Default, Component)]
pub struct SmudPickingCamera;

/// An optional component that provides a custom distance function for precise hit testing.
///
/// When present, this will be used instead of simple bounds-based picking to determine
/// if a point is inside or outside the shape. The function receives an `SdfInput` struct
/// containing position, bounds, and params, matching the shader interface.
#[derive(Component)]
pub struct SmudPickingShape {
    /// The signed distance function. Returns negative values inside the shape,
    /// positive values outside, and zero on the surface.
    /// Takes SdfInput with current position, bounds, and params.
    pub distance_fn: Box<dyn Fn(SdfInput) -> f32 + Send + Sync>,
}

impl SmudPickingShape {
    /// Create a new SDF picking shape with the given distance function.
    pub fn new<F>(distance_fn: F) -> Self
    where
        F: Fn(SdfInput) -> f32 + Send + Sync + 'static,
    {
        Self {
            distance_fn: Box::new(distance_fn),
        }
    }
}

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
            .add_systems(PreUpdate, smud_picking.in_set(PickingSystems::Backend));
    }
}

/// The main picking system that tests pointer intersections with SDF shapes.
#[allow(clippy::type_complexity)]
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
        Option<&SmudPickingShape>,
    )>,
    mut output: MessageWriter<PointerHits>,
) {
    // Collect shapes sorted by depth (back to front for proper ordering)
    let mut sorted_shapes: Vec<_> = shapes
        .iter()
        .filter_map(
            |(entity, shape, transform, visibility, pickable, sdf_shape)| {
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

                Some((entity, shape, transform, pickable, sdf_shape))
            },
        )
        .collect();

    // Sort by Z coordinate (back to front)
    sorted_shapes.sort_by(|(_, _, transform_a, _, _), (_, _, transform_b, _, _)| {
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
        for (entity, shape, shape_transform, pickable, sdf_picking) in &sorted_shapes {
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
            let intersection_point = ray_start + *ray_direction * t;

            // Transform the intersection point to shape local space
            let world_to_shape = shape_transform.affine().inverse();
            let local_point = world_to_shape.transform_point3(intersection_point);

            // Check if the point is within the shape using SDF or bounding box
            let is_hit = if let Some(sdf_shape) = sdf_picking {
                // Use the custom SDF function for precise hit testing with current bounds and params
                let sdf_input = SdfInput {
                    pos: Vec2::new(local_point.x, local_point.y),
                    bounds: shape.bounds.half_size,
                    params: shape.params,
                };
                let distance = (sdf_shape.distance_fn)(sdf_input);
                distance <= 0.0 // Inside or on the surface
            } else {
                // Fall back to bounds-based hit testing
                let half_size = shape.bounds.half_size;
                local_point.x.abs() <= half_size.x && local_point.y.abs() <= half_size.y
            };

            if is_hit {
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
