use crate::{aabb::AxisAlignedBB, obb::OrientedBB, sphere::BSphere, BoundingVolume};
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
#[allow(clippy::type_complexity)]
pub fn update_debug_meshes<T>(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<
        (&'static GlobalTransform, &T, Entity, Option<&Children>),
        (Changed<T>, With<DebugBounds>),
    >,
    mut debug_mesh_query: Query<&mut Handle<Mesh>, With<DebugBoundsMesh>>,
) where
    T: 'static + BoundingVolume + Clone + Send + Sync + std::fmt::Debug,
    Mesh: From<&'static T>,
{
    for (transform, bound_vol, entity, optional_children) in query.iter() {
        let mut updated_existing_child = false;
        if let Some(children) = optional_children {
            for child in children.iter() {
                if let Ok(mut mesh_handle) = debug_mesh_query.get_mut(*child) {
                    let mesh = bound_vol.new_debug_mesh(transform);
                    let new_handle = meshes.add(mesh);
                    *mesh_handle = new_handle;
                    updated_existing_child = true;
                    break;
                }
            }
        }
        // if the entity had a child, we don't need to create a new one
        if !updated_existing_child {
            let mesh_handle = meshes.add(bound_vol.new_debug_mesh(transform));
            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle,
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.0, 1.0, 0.0),
                            unlit: true,
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .insert(DebugBoundsMesh);
            });
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_debug_mesh_visibility<T>(
    mut query: QuerySet<(
        Query<(&Children, &Visible), (With<DebugBounds>, With<T>, Changed<Visible>)>,
        Query<&mut Visible, With<DebugBoundsMesh>>,
    )>,
) where
    T: 'static + BoundingVolume + Clone + Send + Sync,
{
    let child_list: Vec<(Box<Children>, bool)> = query
        .q0()
        .iter()
        .map(|(children, visible)| (Box::new((*children).clone()), visible.is_visible))
        .collect();
    for (children, parent_visible) in child_list.iter() {
        for child in children.iter() {
            if let Ok(mut child_visible) = query.q1_mut().get_mut(*child) {
                child_visible.is_visible = *parent_visible;
            }
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

impl From<&BSphere> for Mesh {
    fn from(sphere: &BSphere) -> Self {
        let radius = sphere.mesh_space_radius();
        let origin = sphere.mesh_space_origin();
        let n_points: i8 = 24;
        let vertices_x0: Vec<[f32; 3]> = (0..n_points)
            .map(|i| {
                let angle = i as f32 * 2.0 * std::f32::consts::PI / (n_points as f32);
                [
                    0.0,
                    angle.sin() * radius + origin.y,
                    angle.cos() * radius + origin.z,
                ]
            })
            .collect();
        let vertices_y0: Vec<[f32; 3]> = (0..n_points)
            .map(|i| {
                let angle = i as f32 * 2.0 * std::f32::consts::PI / (n_points as f32);
                [
                    angle.cos() * radius + origin.x,
                    0.0,
                    angle.sin() * radius + origin.z,
                ]
            })
            .collect();
        let vertices_z0: Vec<[f32; 3]> = (0..n_points)
            .map(|i| {
                let angle = i as f32 * 2.0 * std::f32::consts::PI / (n_points as f32);
                [
                    angle.cos() * radius + origin.x,
                    angle.sin() * radius + origin.y,
                    0.0,
                ]
            })
            .collect();
        let vertices = [vertices_x0, vertices_y0, vertices_z0].concat();
        let indices_single: Vec<u32> = (0..n_points * 2)
            .map(|i| {
                let result = (i as u32 + 1) / 2;
                if result == n_points as u32 {
                    0
                } else {
                    result
                }
            })
            .collect();
        let indices = Indices::U32(
            [
                indices_single
                    .iter()
                    .map(|&index| index + n_points as u32)
                    .collect(),
                indices_single
                    .iter()
                    .map(|&index| index + 2 * n_points as u32)
                    .collect(),
                indices_single,
            ]
            .concat(),
        );
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vertices.clone());
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertices);
        mesh.set_indices(Some(indices));
        mesh
    }
}
