mod boxes;
mod sphere;

use bevy::prelude::*;
use boxes::*;
use sphere::*;

pub struct BoundingVolumePlugin;
impl Plugin for BoundingVolumePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(stage::POST_UPDATE, update_bounding_volumes.system());
    }
}

/// The set of bounding volume types to choose from
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BoundingVolume {
    BoundingSphere(Option<BoundingSphere>),
    AxisAlignedBoundingBox(Option<AxisAlignedBoundingBox>),
    OrientedBoundingBox(Option<OrientedBoundingBox>), // Not Implemented
    DiscreteOrientedPolytope,                         // Not Implemented
    ConvexHull,                                       // Not Implemented
}

/// Iterates through all entities with [BoundingVolume]s, and updates them if the volume was just
/// added, or the entity's mesh has changed.
pub fn update_bounding_volumes(
    meshes: Res<Assets<Mesh>>,
    mut new_or_changed_vols_query: Query<
        (&mut BoundingVolume, &Handle<Mesh>),
        Or<(Added<BoundingVolume>, Changed<Handle<Mesh>>)>,
    >,
) {
    for (mut bound_vol, mesh_handle) in &mut new_or_changed_vols_query.iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            match *bound_vol {
                BoundingVolume::BoundingSphere(ref mut vol) => {
                    *vol = Some(BoundingSphere::from(mesh))
                }
                BoundingVolume::AxisAlignedBoundingBox(ref mut vol) => {
                    *vol = Some(AxisAlignedBoundingBox::from(mesh))
                }
                BoundingVolume::OrientedBoundingBox(ref mut vol) => {
                    *vol = Some(OrientedBoundingBox::from(mesh))
                }
                _ => continue,
            }
        } else {
            continue;
        }
    }
}
