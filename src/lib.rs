//mod boxes;
//mod debug;
mod sphere;

use std::marker::PhantomData;

use bevy::prelude::*;
//use boxes::*;
//use debug::*;
pub use sphere::BoundingSphere;
use sphere::*;

//pub use debug::BoundingVolumeDebug;

pub struct BoundingVolumePlugin;
impl Plugin for BoundingVolumePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app;
    }
}

pub struct BoundingVolumeDebugPlugin;
impl Plugin for BoundingVolumeDebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app;
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

pub trait IsBoundingVolume {
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self;
}

#[derive(Bundle)]
pub struct BoundingVolumeBundle<T: 'static + IsBoundingVolume + Send + Sync> {
    pub vol: T,
    pub transform: GlobalTransform,
}

/// Use generics to spawn a child entity with component type T
pub fn spawn_bounding_volumes<T: 'static + IsBoundingVolume + Send + Sync>(
    commands: &mut Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<(&Handle<Mesh>, &GlobalTransform, Entity), Added<BoundingVolume<T>>>,
) {
    for (handle, transform, entity) in query.iter() {
        let mesh = meshes
            .get(handle)
            .expect("Bad mesh handle component on BoundingVolume entity");
        let new_vol = T::new(mesh, transform);
        commands.set_current_entity(entity);
        commands.with_children(|parent| {
            parent.spawn(BoundingVolumeBundle {
                vol: new_vol,
                transform: GlobalTransform::default(),
            });
        });
    }
}
