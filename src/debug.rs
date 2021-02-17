use crate::BoundingVolume;
use bevy::prelude::*;

/// Marks an entity that should have a mesh added as a child to represent the mesh's bounding volume.
pub struct BoundingVolumeDebug;

/// Marks the debug bounding volume mesh, which exists as a child of a [BoundingVolumeDebug] entity
pub struct BoundingVolumeDebugMesh;

/// Updates existing debug meshes, and creates new debug meshes on entities with a bounding volume
/// component marked with [BoundingVolumeDebug] and no existing debug mesh.
pub fn update_debug_meshes<T: 'static + BoundingVolume + Send + Sync>(
    commands: &mut Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<
        (&GlobalTransform, &T, Entity, Option<&Children>),
        (Changed<T>, With<BoundingVolumeDebug>),
    >,
    mut debug_mesh_query: Query<&mut Handle<Mesh>, With<BoundingVolumeDebugMesh>>,
) {
    for (transform, bound_vol, entity, optional_children) in query.iter() {
        let mut updated_existing_child = false;
        if let Some(children) = optional_children {
            for child in children.iter() {
                if let Ok(mut mesh_handle) = debug_mesh_query.get_mut(*child) {
                    let new_handle = meshes.add(bound_vol.new_debug_mesh(transform));
                    *mesh_handle = new_handle;
                    updated_existing_child = true;
                }
            }
        }
        // if the entity had a child, we don't need to create a new one
        if !updated_existing_child {
            let mesh_handle = meshes.add(bound_vol.new_debug_mesh(transform));
            commands.set_current_entity(entity);
            commands.with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: mesh_handle,
                        material: materials.add(StandardMaterial {
                            albedo: Color::rgb(1.0, 0.0, 1.0),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .with(BoundingVolumeDebugMesh);
            });
        }
    }
}
