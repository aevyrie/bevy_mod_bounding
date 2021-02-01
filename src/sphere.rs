use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;

use crate::{BoundingVolume, IsBoundingVolume};

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug, Clone)]
pub struct BoundingSphere {
    origin: Vec3,
    radius: f32,
}

/// Updates the definition of
pub fn bounding_sphere_update_system(
    meshes: Res<Assets<Mesh>>,
    mut sphere_query: Query<
        (&mut BoundingSphere, &GlobalTransform, &Handle<Mesh>),
        Or<(Changed<GlobalTransform>, Changed<Handle<Mesh>>)>,
    >,
) {
    for (mut bounding_sphere, transform, handle) in sphere_query.iter_mut() {
        let mesh = meshes
            .get(handle)
            .expect("Bounding volume had bad mesh handle");
        *bounding_sphere = BoundingSphere::new(mesh, transform);
    }
}

impl BoundingSphere {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
}

impl IsBoundingVolume for BoundingSphere {
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self {
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding sphere generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float3(positions) => positions
                    .iter()
                    .map(|coordinates| transform.mul_vec3(Vec3::from(*coordinates)))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        let point_x = vertices[0];
        // Find point y, the point furthest from point x
        let point_y = vertices.iter().fold(point_x, |acc, x| {
            if x.distance(point_x) >= acc.distance(point_x) {
                *x
            } else {
                acc
            }
        });
        // Find point z, the point furthest from point y
        let point_z = vertices.iter().fold(point_y, |acc, x| {
            if x.distance(point_y) >= acc.distance(point_y) {
                *x
            } else {
                acc
            }
        });
        // Construct a bounding sphere using these two points as the poles
        let mut sphere = BoundingSphere {
            origin: point_y.lerp(point_z, 0.5),
            radius: point_y.distance(point_z) / 2.0,
        };
        // Iteratively adjust sphere until it encloses all points
        loop {
            // Find the furthest point from the origin
            let point_n = vertices.iter().fold(point_x, |acc, x| {
                if x.distance(sphere.origin) >= acc.distance(sphere.origin) {
                    *x
                } else {
                    acc
                }
            });
            // If the furthest point is outside the sphere, we need to adjust it
            let point_dist = point_n.distance(sphere.origin);
            if point_dist > sphere.radius {
                let radius_new = (sphere.radius + point_dist) / 2.0;
                let lerp_ratio = (point_dist - radius_new) / point_dist;
                sphere = BoundingSphere {
                    origin: sphere.origin.lerp(point_n, lerp_ratio),
                    radius: radius_new,
                };
            } else {
                return sphere;
            }
        }
    }
}
