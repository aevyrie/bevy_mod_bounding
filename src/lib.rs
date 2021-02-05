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

//pub use debug::BoundingVolumeDebug;

pub struct BoundingVolumePlugin;
impl Plugin for BoundingVolumePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(PRE_UPDATE, spawn_bounds::<BSphere>.system())
            .add_system_to_stage(PRE_UPDATE, spawn_bounds::<AxisAlignedBB>.system())
            .add_system_to_stage(PRE_UPDATE, spawn_bounds::<OrientedBB>.system())
            .add_system_to_stage(POST_UPDATE, BSphere::update.system())
            .add_system_to_stage(POST_UPDATE, AxisAlignedBB::update.system())
            .add_system_to_stage(POST_UPDATE, OrientedBB::update.system())
            .add_system_to_stage(POST_UPDATE, debug::update_debug_meshes::<BSphere>.system())
            .add_system_to_stage(POST_UPDATE, update_debug_meshes::<AxisAlignedBB>.system())
            .add_system_to_stage(POST_UPDATE, update_debug_meshes::<OrientedBB>.system());
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
}

/// Marks new BoundingVolumes that are awaiting their mesh to load
pub struct LoadingMesh;

/// Use generics to spawn a child entity with component type T
pub fn spawn_bounds<T: 'static + IsBoundingVolume + Send + Sync>(
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
                println!("mesh loaded");
            }
            None => {
                commands.set_current_entity(entity);
                commands.with(LoadingMesh);
            }
        }
    }
}
