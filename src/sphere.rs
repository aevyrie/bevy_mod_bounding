use crate::BoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;

/// Defines a bounding sphere with a radius and an origin at the center.
#[derive(Debug, Clone, Default, Component)]
pub struct BSphere {
    /// Origin of the sphere in mesh space. The intent is that the bounding volume will be queried
    /// along with its [GlobalTransform], so the origin of the sphere will be transformed to the
    /// world position of the mesh, and the radius can be used to determine the bounding volume.
    mesh_space_origin: Vec3,
    /// Radius of the sphere that bounds the mesh, in mesh space.
    mesh_space_radius: f32,
}
impl BSphere {
    /// Given the current [GlobalTransform] of the bounded mesh, returns the central origin of the
    /// sphere that bounds the mesh in world space.
    pub fn origin(&self, transform: GlobalTransform) -> Vec3 {
        self.mesh_space_origin + transform.translation
    }
    /// Given the current [GlobalTransform] of the bounded mesh, returns the radius of the sphere
    /// that bounds the mesh in world space.
    pub fn radius(&self, transform: &GlobalTransform) -> f32 {
        self.mesh_space_radius * transform.scale.max_element()
    }
    /// Get a reference to the b sphere's mesh space origin.
    pub fn mesh_space_origin(&self) -> &Vec3 {
        &self.mesh_space_origin
    }
    /// Get a reference to the b sphere's mesh space radius.
    pub fn mesh_space_radius(&self) -> &f32 {
        &self.mesh_space_radius
    }
}

/// Create a valid boundary sphere from a mesh and globaltransform.
impl BoundingVolume for BSphere {
    fn new(mesh: &Mesh, _transform: &GlobalTransform) -> Self {
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding sphere generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float32x3(positions) => positions
                    .iter()
                    .map(|coordinates| Vec3::from(*coordinates))
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
        let mut sphere = BSphere {
            mesh_space_origin: point_y.lerp(point_z, 0.5),
            mesh_space_radius: point_y.distance(point_z) / 2.0,
        };
        // Iteratively adjust sphere until it encloses all points
        loop {
            // Find the furthest point from the origin
            let point_n = vertices.iter().fold(point_x, |acc, x| {
                if x.distance(sphere.mesh_space_origin) >= acc.distance(sphere.mesh_space_origin) {
                    *x
                } else {
                    acc
                }
            });
            // If the furthest point is outside the sphere, we need to adjust it
            let point_dist = point_n.distance(sphere.mesh_space_origin);
            if point_dist > sphere.mesh_space_radius {
                let radius_new = (sphere.mesh_space_radius + point_dist) / 2.0;
                let lerp_ratio = (point_dist - radius_new) / point_dist;
                sphere = BSphere {
                    mesh_space_origin: sphere.mesh_space_origin.lerp(point_n, lerp_ratio),
                    mesh_space_radius: radius_new,
                };
            } else {
                return sphere;
            }
        }
    }

    /// Generate a debug mesh, and apply the inverse transform. Because the debug mesh is a child,
    /// the transform of the parent will be applied to it. This needs to be negated so the bounding
    /// circle debug mesh isn't warped.
    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh {
        let mut mesh = Mesh::from(self);
        let inverse_transform = Transform::from_matrix(
            Mat4::from_scale_rotation_translation(Vec3::ONE, transform.rotation, Vec3::ZERO)
                .inverse(),
        );
        match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match vertex_values {
                VertexAttributeValues::Float32x3(ref mut positions) => {
                    *positions = positions
                        .iter()
                        .map(|coordinates| {
                            inverse_transform.mul_vec3(Vec3::from(*coordinates)).into()
                        })
                        .collect()
                }
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        mesh
    }

    fn update_on_transform_change(&self, mesh: &Mesh, transform: &GlobalTransform) -> Option<Self> {
        Some(Self::new(mesh, transform))
    }

    fn outside_plane(
        &self,
        bound_vol_position: &GlobalTransform,
        point: Vec3,
        normal: Vec3,
    ) -> bool {
        normal.dot(self.origin(*bound_vol_position)) + -normal.dot(point)
            - self.radius(bound_vol_position)
            > 0.0
    }
}
