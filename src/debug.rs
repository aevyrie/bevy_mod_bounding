use crate::{BoundingVolumeDebug, IsBoundingVolume};
use bevy::prelude::*;

/// Marks the debug bounding volume mesh of a BoundingVolumeDebug entity
pub struct BoundingVolumeDebugMesh;

pub fn update_debug_meshes<T: 'static + IsBoundingVolume + Send + Sync>(
    commands: &mut Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<
        (&GlobalTransform, &T, Entity, Option<&Children>),
        (Changed<T>, With<BoundingVolumeDebug>),
    >,
    debug_mesh_query: Query<&Handle<Mesh>, With<BoundingVolumeDebugMesh>>,
) {
    for (transform, bound_vol, entity, optional_children) in query.iter() {
        let mut updated_existing_child = false;
        if let Some(children) = optional_children {
            for child in children.iter() {
                if let Ok(mesh_handle) = debug_mesh_query.get(*child) {
                    println!("update debug");
                    *meshes
                        .get_mut(mesh_handle)
                        .expect("Bad handle in bounding debug mush") =
                        bound_vol.new_debug_mesh(transform);
                    updated_existing_child = true;
                }
            }
        }
        if !updated_existing_child {
            println!("debug spawn");
            let mesh_handle = meshes.add(bound_vol.new_debug_mesh(transform));
            commands.set_current_entity(entity);
            commands.with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: mesh_handle,
                        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                        ..Default::default()
                    })
                    .with(BoundingVolumeDebugMesh);
            });
        }
    }
}
