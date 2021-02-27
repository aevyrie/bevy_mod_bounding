use crate::{AxisAlignedBB, BSphere, BoundingVolume, OrientedBB};
use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};

/// Marks an entity that should have a mesh added as a child to represent the mesh's bounding volume.
pub struct DebugBounds;

/// Marks the debug bounding volume mesh, which exists as a child of a [BoundingVolumeDebug] entity
pub struct DebugBoundsMesh;

/// Updates existing debug meshes, and creates new debug meshes on entities with a bounding volume
/// component marked with [BoundingVolumeDebug] and no existing debug mesh.
pub fn update_debug_meshes<T>(
    commands: &mut Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<
        (&GlobalTransform, &T, Entity, Option<&Children>),
        (Changed<T>, With<DebugBounds>),
    >,
    mut debug_mesh_query: Query<&mut Handle<Mesh>, With<DebugBoundsMesh>>,
) where
    T: 'static + BoundingVolume + Clone + Send + Sync,
    Mesh: From<&'static T>,
{
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
                            albedo: Color::rgb(0.0, 1.0, 0.0),
                            unlit: true,
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .with(DebugBoundsMesh);
            });
        }
    }
}

impl From<&AxisAlignedBB> for Mesh {
    fn from(aabb: &AxisAlignedBB) -> Self {
        /*
              (2)-----(3)               Y
               | \     | \              |
               |  (1)-----(0) MAX       o---X
               |   |   |   |             \
          MIN (6)--|--(7)  |              Z
                 \ |     \ |
                  (5)-----(4)
        */
        let vertices: Vec<[f32; 3]> = aabb
            .vertices_mesh_space()
            .iter()
            .map(|vert| [vert.x, vert.y, vert.z])
            .collect();

        let indices = Indices::U32(vec![
            0, 1, 1, 2, 2, 3, 3, 0, // Top ring
            4, 5, 5, 6, 6, 7, 7, 4, // Bottom ring
            0, 4, 1, 5, 2, 6, 3, 7, // Verticals
        ]);

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertices);
        mesh.set_indices(Some(indices));
        mesh
    }
}

impl From<&OrientedBB> for Mesh {
    fn from(obb: &OrientedBB) -> Self {
        /*
              (2)-----(3)               Y
               | \     | \              |
               |  (1)-----(0) MAX       o---X
               |   |   |   |             \
          MIN (6)--|--(7)  |              Z
                 \ |     \ |
                  (5)-----(4)
        */
        let vertices: Vec<[f32; 3]> = obb
            .vertices_mesh_space()
            .iter()
            .map(|vert| [vert.x, vert.y, vert.z])
            .collect();

        let indices = Indices::U32(vec![
            0, 1, 1, 2, 2, 3, 3, 0, // Top ring
            4, 5, 5, 6, 6, 7, 7, 4, // Bottom ring
            0, 4, 1, 5, 2, 6, 3, 7, // Verticals
        ]);

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertices);
        mesh.set_indices(Some(indices));
        mesh
    }
}
