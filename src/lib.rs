mod boxes;
mod debug;
mod sphere;

use bevy::prelude::*;
use boxes::*;
use debug::*;
use sphere::*;

pub use debug::BoundingVolumeDebug;

pub struct BoundingVolumePlugin;
impl Plugin for BoundingVolumePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, update_bounding_volumes.system());
    }
}

pub struct BoundingVolumeDebugPlugin;
impl Plugin for BoundingVolumeDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, spawn_debug_meshes.system())
            .add_system_to_stage(stage::UPDATE, create_new_debug_meshes.system())
            .add_system_to_stage(stage::POST_UPDATE, update_debug_meshes.system())
            .add_system_to_stage(stage::POST_UPDATE, update_debug_mesh_transforms.system());
    }
}

/// The set of bounding volume types to choose from
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BoundingVolume {
    BoundingSphere(Option<BoundingSphere>),
    AxisAlignedBoundingBox(Option<AxisAlignedBoundingBox>),
    OrientedBoundingBox(Option<OrientedBoundingBox>),
    DiscreteOrientedPolytope, // Not Implemented
    ConvexHull,               // Not Implemented
}
impl BoundingVolume {
    pub fn initialized(&self) -> bool {
        match self {
            BoundingVolume::BoundingSphere(vol) => vol.is_some(),
            BoundingVolume::AxisAlignedBoundingBox(vol) => vol.is_some(),
            BoundingVolume::OrientedBoundingBox(vol) => vol.is_some(),
            _ => false,
        }
    }
}

/// Used to mark new bounding volumes that are waiting on meshes to load or bounding volumes to be
/// computed.
#[derive(Default)]
pub struct BoundLoading;

#[derive(Bundle)]
pub struct BoundingVolumeBundle {
    pub bound_vol: BoundingVolume,
    loading_marker: BoundLoading,
}
impl BoundingVolumeBundle {
    pub fn new(bound_vol: BoundingVolume) -> Self {
        BoundingVolumeBundle {
            bound_vol,
            loading_marker: BoundLoading,
        }
    }
}

/// Iterates through all entities with [BoundingVolume]s, and updates them if the volume was just
/// added, or the entity's mesh has changed.
pub fn update_bounding_volumes(
    commands: &mut Commands,
    meshes: Res<Assets<Mesh>>,
    mut vols_query: QuerySet<(
        Query<
            (&mut BoundingVolume, &Handle<Mesh>, Entity, &GlobalTransform),
            Or<(
                Changed<BoundingVolume>,
                Changed<Handle<Mesh>>,
                With<BoundLoading>,
            )>,
        >,
        Query<
            (&mut BoundingVolume, &Handle<Mesh>, &GlobalTransform),
            Or<(Changed<GlobalTransform>,)>,
        >,
    )>,
) {
    for (mut bound_vol, mesh_handle, entity, transform) in &mut vols_query.q0_mut().iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            match *bound_vol {
                BoundingVolume::BoundingSphere(ref mut vol) => {
                    *vol = Some(BoundingSphere::from(mesh))
                }
                BoundingVolume::AxisAlignedBoundingBox(ref mut vol) => {
                    *vol = Some(AxisAlignedBoundingBox::new(mesh, transform))
                }
                BoundingVolume::OrientedBoundingBox(ref mut vol) => {
                    *vol = Some(OrientedBoundingBox::from(mesh))
                }
                _ => (),
            }
            commands.remove_one::<BoundLoading>(entity);
        } else {
            commands.set_current_entity(entity);
            commands.with(BoundLoading);
        }
    }
    for (mut bound_vol, mesh_handle, transform) in &mut vols_query.q1_mut().iter_mut() {
        if let BoundingVolume::AxisAlignedBoundingBox(ref mut vol) = *bound_vol {
            if let Some(mesh) = meshes.get(mesh_handle) {
                *vol = Some(AxisAlignedBoundingBox::new(mesh, transform))
            }
        }
    }
}
