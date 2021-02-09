mod axis_aligned_box;
mod debug;
mod oriented_box;
mod sphere;

use bevy::prelude::{stage::*, *};
use debug::update_debug_meshes;
use std::marker::PhantomData;

pub use axis_aligned_box::AxisAlignedBB;
pub use oriented_box::OrientedBB;
pub use sphere::BSphere;

#[derive(Default)]
pub struct BoundingVolumePlugin<T: IsBoundingVolume> {
    marker: std::marker::PhantomData<T>,
}
impl<T: 'static + Send + Sync + IsBoundingVolume> Plugin for BoundingVolumePlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(PRE_UPDATE, spawn::<T>.system())
            .add_system_to_stage(POST_UPDATE, update::<T>.system())
            .add_system_to_stage(POST_UPDATE, update_debug_meshes::<T>.system());
    }
}

///
#[derive(Debug, Clone)]
pub struct BoundingVolume<T: IsBoundingVolume + Send + Sync>(PhantomData<T>);
impl<T: IsBoundingVolume + Send + Sync> Default for BoundingVolume<T> {
    fn default() -> Self {
        BoundingVolume(PhantomData::default())
    }
}

pub struct BoundingVolumeDebug;

pub trait IsBoundingVolume {
    /// Initializes a valid bounding volume given a [Mesh] and [GlobalTransform].
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self;
    /// Generate a [Mesh] from the [BoundingVolume]'s own definition.
    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh;
    /// Generate a bounding volume when the [BoundingVolume] changes.
    fn update_on_mesh_change(&self, mesh: &Mesh, transform: &GlobalTransform) -> Self;
    /// Generate a bounding volume when the [Mesh] changes.
    fn update_on_transform_change(&self, mesh: &Mesh, transform: &GlobalTransform) -> Self;
}

/// Marks new BoundingVolumes that are awaiting their mesh to load
pub struct LoadingMesh;

/// Use generics to spawn a child entity with component type T
pub fn spawn<T: 'static + IsBoundingVolume + Send + Sync>(
    commands: &mut Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<
        (&Handle<Mesh>, &GlobalTransform, Entity),
        (
            Or<(Added<BoundingVolume<T>>, With<LoadingMesh>)>,
            Without<T>,
            With<BoundingVolume<T>>,
        ),
    >,
) {
    for (handle, transform, entity) in query.iter() {
        match meshes.get(handle) {
            Some(mesh) => {
                commands.set_current_entity(entity);
                commands.with(T::new(mesh, transform));
                commands.remove_one::<LoadingMesh>(entity);
                commands.remove_one::<BoundingVolume<T>>(entity);
            }
            None => {
                commands.set_current_entity(entity);
                commands.with(LoadingMesh);
            }
        }
    }
}

fn update<T: 'static + IsBoundingVolume + Send + Sync>(
    meshes: Res<Assets<Mesh>>,
    mut query: QuerySet<(
        Query<(&mut T, &GlobalTransform, &Handle<Mesh>, Entity), Changed<GlobalTransform>>,
        Query<(&mut T, &GlobalTransform, &Handle<Mesh>, Entity), Changed<Handle<Mesh>>>,
    )>,
) {
    let mut changed_bounding_vols: Vec<Entity> = Vec::new();
    for (mut bounding_vol, transform, handle, entity) in query.q0_mut().iter_mut() {
        let mesh = meshes
            .get(handle)
            .expect("Bounding volume had bad mesh handle");
        *bounding_vol = bounding_vol.update_on_transform_change(mesh, transform);
        changed_bounding_vols.push(entity);
    }
    for (mut bounding_vol, transform, handle, entity) in query.q1_mut().iter_mut() {
        if !changed_bounding_vols.contains(&entity) {
            let mesh = meshes
                .get(handle)
                .expect("Bounding volume had bad mesh handle");
            *bounding_vol = bounding_vol.update_on_mesh_change(mesh, transform);
        }
    }
}
