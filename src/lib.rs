mod aabb;
mod debug;
mod obb;
mod sphere;

use bevy::{prelude::*, transform::TransformSystem};
use debug::update_debug_meshes;
use std::marker::PhantomData;
use std::{borrow::Cow, fmt::Debug};

pub use aabb::AxisAlignedBB;
pub use debug::BoundingVolumeDebug;
pub use obb::OrientedBB;
pub use sphere::BSphere;

#[derive(Default)]
pub struct BoundingVolumePlugin<T: BoundingVolume> {
    marker: std::marker::PhantomData<T>,
}

/// A plugin that provides functionality for generating and updating bounding volumes for meshes.
impl<T> Plugin for BoundingVolumePlugin<T>
where
    T: 'static + Send + Sync + BoundingVolume,
{
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::PreUpdate, spawn::<T>.system())
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update::<T>
                    .system()
                    .after(TransformSystem::TransformPropagate)
                    .label(Cow::Owned(format!(
                        "update_boundvols_{}",
                        std::any::type_name::<T>()
                    ))),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_debug_meshes::<T>.system().after(Cow::Owned(format!(
                    "update_boundvols_{}",
                    std::any::type_name::<T>()
                ))),
            );
    }
}

/// Marks an entity to have a bounding volume generated. This entity should have a [Mesh]
/// component. A bounding volume component of type T will be computed and added to the entity once
/// the aforementioned mesh has loaded and can be read. This ensures that bounding volume
/// components are always valid when queried, and at worst case can only be out of date if queried
/// in a frame before the bounding volume update system is run.
#[derive(Debug, Clone)]
pub struct AddBoundingVolume<T: BoundingVolume + Send + Sync>(PhantomData<T>);

impl<T: BoundingVolume + Send + Sync> Default for AddBoundingVolume<T> {
    fn default() -> Self {
        AddBoundingVolume(PhantomData::default())
    }
}

/// A [BoundingVolume] stores its properties in mesh space to maximize precision. Because some types
/// of bounding volume must be recomputed if the mesh is scaled or rotated, this trait calls an
/// update function depending on whether the mesh or transform has updated.
pub trait BoundingVolume {
    /// Initializes a valid bounding volume given a [Mesh] and [GlobalTransform].
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self;
    /// Generate a debug [Mesh] representing the bounding volume from a [BoundingVolume].
    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh;
    /// This function is only called when only the entity's [GlobalTransform] has changed. Only
    /// some types of bounding volume need to be recomputed in this case.
    fn update_on_transform_change(&mut self, mesh: &Mesh, transform: &GlobalTransform);
    /// Returns true iff the bounding mesh is entirely on the outside of the supplied plane.
    /// "Outside" is the direction that the plane normal points to.
    fn outside_plane(
        &self,
        bound_vol_position: &GlobalTransform,
        point: Vec3,
        normal: Vec3,
    ) -> bool;
}

/// Spawns a new [BoundingVolume], replacing the [AddBoundingVolume] marker component on the
/// entity. This new BoundingVolume is fully initialized and will be kept up to date with the
/// `update()` system.
pub fn spawn<T: 'static + BoundingVolume + Send + Sync>(
    commands: &mut Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<(&Handle<Mesh>, &GlobalTransform, Entity), With<AddBoundingVolume<T>>>,
) {
    for (handle, transform, entity) in query.iter() {
        if let Some(mesh) = meshes.get(handle) {
            commands.set_current_entity(entity);
            commands.with(T::new(mesh, transform));
            commands.remove_one::<AddBoundingVolume<T>>(entity);
        }
    }
}

/// Updates [BoundingVolume]s when their meshes or [GlobalTransform]s are changed. If an entity's
/// mesh has changed, triggering a bounding volume update, the update function will won't update it
/// a second time if the transform has also changed.
fn update<T: 'static + BoundingVolume + Send + Sync>(
    meshes: Res<Assets<Mesh>>,
    changed_mesh_query: Query<Entity, Changed<Handle<Mesh>>>,
    changed_transform_query: Query<Entity, Changed<GlobalTransform>>,
    mut bound_vol_query: Query<(&mut T, &GlobalTransform, &Handle<Mesh>)>,
) {
    for entity in changed_mesh_query.iter() {
        if let Ok((mut bounding_vol, transform, handle)) = bound_vol_query.get_mut(entity) {
            let mesh = meshes
                .get(handle)
                .expect("Bounding volume had bad mesh handle");
            *bounding_vol = T::new(mesh, transform);
        }
    }
    for entity in changed_transform_query.iter() {
        // Only process entities that haven't already been updated.
        if let Err(_) = changed_mesh_query.get(entity) {
            if let Ok((mut bounding_vol, transform, handle)) = bound_vol_query.get_mut(entity) {
                let mesh = meshes
                    .get(handle)
                    .expect("Bounding volume had bad mesh handle");
                bounding_vol.update_on_transform_change(mesh, transform);
            }
        }
    }
}
