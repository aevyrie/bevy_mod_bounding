use crate::{BoundLoading, BoundingVolume};
use bevy::prelude::*;

/// Marks an entity to get a bounding mesh.
///
/// # Requirements
///
/// The entity must also have a [BoundingVolume] component.
#[derive(Default)]


/// Used to mark the actual debug mesh, and records the entity of the associated mesh
pub struct BoundingMesh {
    bound_vol_entity: Entity,
    mesh_transform: Transform,
}

pub struct BoundingMeshNotSpawned;

#[derive(Bundle)]
struct BoundingMeshBundle {
    pub bound_mesh: BoundingMesh,
    pub loading_marker: BoundingMeshNotSpawned,
}

pub fn spawn_debug_meshes(
    commands: &mut Commands,
    new_vols_query: Query<Entity, (Without<BoundLoading>, With<BoundingVolume>)>,
    bound_mesh_query: Query<&BoundingMesh>,
) {
    'outer: for entity in new_vols_query.iter() {
        for bound_mesh in bound_mesh_query.iter() {
            if bound_mesh.bound_vol_entity == entity {
                // if a debug mesh for this entity already exists, exit
                break 'outer;
            }
        }
        commands.spawn(BoundingMeshBundle {
            bound_mesh: BoundingMesh {
                bound_vol_entity: entity,
                mesh_transform: Transform::default(),
            },
            loading_marker: BoundingMeshNotSpawned,
        });
    }
}

/// Spawns new entities with bounding meshes that are marked as not yet spawned
pub fn create_new_debug_meshes(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // For creating new meshes
    mut loading_bound_mesh_query: Query<(&mut BoundingMesh, Entity), With<BoundingMeshNotSpawned>>,
    all_volumes_query: Query<(&BoundingVolume, &GlobalTransform)>,
) {
    let debug_material = &materials.add(StandardMaterial {
        albedo: Color::rgb(1.0, 0.0, 1.0),
        unlit: false,
        ..Default::default()
    });

    // Go through each bounding mesh that is still loading, compute the bounding mesh, then remove
    // loading flag.
    for (mut bound_mesh, bound_mesh_entity) in loading_bound_mesh_query.iter_mut() {
        let (bound_vol, vol_transform) = all_volumes_query
            .get(bound_mesh.bound_vol_entity)
            .expect("Invalid entity stored in BoundingMesh");
        let (new_mesh, new_transform) =
            compute_bound_mesh(bound_vol).expect("Uninitialized bounding volume");
        bound_mesh.mesh_transform = new_transform;
        let mesh_handle = meshes.add(new_mesh);
        let new_compound_transform = Transform::from(*vol_transform) * new_transform;
        commands.set_current_entity(bound_mesh_entity);
        commands.with_bundle(PbrBundle {
            mesh: mesh_handle,
            material: debug_material.clone(),
            transform: new_compound_transform,
            ..Default::default()
        });
        commands.remove_one::<BoundingMeshNotSpawned>(bound_mesh_entity);
        info!("New: {:?}", new_transform);
        info!("Compound: {:?}", new_compound_transform);
    }
}

/// Updates debug bounding meshes to match their bounding volume when the bounding volume changes.
pub fn update_debug_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    // For updating meshes when the volume changes
    mut all_bound_mesh_query: Query<
        (&mut BoundingMesh, &Handle<Mesh>, &mut GlobalTransform),
        Without<BoundingMeshNotSpawned>,
    >,
    changed_volumes: Query<
        (&BoundingVolume, Entity, &GlobalTransform),
        (Changed<BoundingVolume>, With<BoundingVolumeDebug>),
    >,
) {
    // Go through all bound vols that have changed (due to some parent mesh change), and
    // recalculate the bounding mesh
    for (bound_vol, vol_entity, vol_transform) in changed_volumes.iter() {
        for (mut bound_mesh, handle, mut old_transform) in all_bound_mesh_query.iter_mut() {
            if bound_mesh.bound_vol_entity != vol_entity {
                continue;
            }
            let old_mesh = meshes
                .get_mut(handle)
                .expect("Bounding mesh handle is invalid");
            let (new_mesh, new_transform) =
                compute_bound_mesh(bound_vol).expect("Uninitialized bounding volume");
            *old_mesh = new_mesh;
            bound_mesh.mesh_transform = new_transform;
            *old_transform = *vol_transform * GlobalTransform::from(new_transform);
        }
    }
}

pub fn update_debug_mesh_transforms(
    mut all_bound_mesh_query: Query<
        (&BoundingMesh, &mut GlobalTransform),
        Without<BoundingMeshNotSpawned>,
    >,
    changed_vol_transform_query: Query<
        (Entity, &GlobalTransform),
        (With<BoundingVolumeDebug>, Changed<GlobalTransform>),
    >,
) {
    // Updated any bounding mesh's transforms whose bounding volume's transform has changed to
    // match.
    for (vol_entity, vol_transform) in changed_vol_transform_query.iter() {
        for (bound_mesh, mut mesh_transform) in all_bound_mesh_query.iter_mut() {
            if bound_mesh.bound_vol_entity != vol_entity {
                continue;
            }
            *mesh_transform = *vol_transform * GlobalTransform::from(bound_mesh.mesh_transform);
        }
    }
}

fn compute_bound_mesh(volume: &BoundingVolume) -> Option<(Mesh, Transform)> {
    match volume {
        BoundingVolume::BoundingSphere(vol) => {
            let volume = match vol {
                Some(vol) => vol,
                None => return None,
            };
            let new_mesh = Mesh::from(shape::Icosphere {
                radius: volume.radius(),
                ..Default::default()
            });
            let transform = Transform::from_translation(volume.origin());
            Some((new_mesh, transform))
        }
        BoundingVolume::AxisAlignedBoundingBox(vol) => {
            let volume = match vol {
                Some(vol) => vol,
                None => return None,
            };
            let new_mesh = Mesh::from(shape::Box::new(
                volume.dimensions().x,
                volume.dimensions().y,
                volume.dimensions().z,
            ));
            dbg!(volume.origin());
            dbg!(volume.dimensions());
            let transform = Transform::from_translation(
                volume
                    .origin()
                    .lerp(volume.origin() + volume.dimensions(), 0.5),
            );
            Some((new_mesh, transform))
        }
        BoundingVolume::OrientedBoundingBox(vol) => {
            let volume = match vol {
                Some(vol) => vol,
                None => return None,
            };
            let new_mesh = Mesh::from(shape::Box::new(
                volume.dimensions().x,
                volume.dimensions().y,
                volume.dimensions().z,
            ));
            let translate = volume
                .origin()
                .lerp(volume.origin() + volume.dimensions(), 0.5);
            let matrix = Mat4::from_rotation_translation(volume.orientation(), translate);
            let transform = Transform::from_matrix(matrix);
            Some((new_mesh, transform))
        }
        _ => None,
    }
}
