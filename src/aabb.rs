use crate::BoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;

/// Defines an axis-aligned bounding box in mesh space - that is - the bounding box is located at
/// the mesh's origin, but the current [GlobalTransform] has been used to rotate and scale the mesh
/// to compute a valid AABB. This reduces float error when the mesh is located far from the origin.
#[derive(Debug, Clone, Default, Component)]
pub struct Aabb {
    /// The coordinates of the point located at the minimum x, y, and z coordinate. This can also
    /// be thought of as the length of the -x, -y, -z axes that extend from the origin and touch
    /// the inside of the bounding box faces.
    minimums: Vec3,
    /// The coordinates of the point located at the maximum x, y, and z coordinate. This can also
    /// be thought of as the length of the +x, +y, +z axes that extend from the origin and touch
    /// the inside of the bounding box faces.
    maximums: Vec3,
}
impl Aabb {
    /// Returns the distance from the origin of the mesh to the negative extents of the bounding
    /// box in world space, aligned with the world axes. I.e.: distance from the mesh origin to the
    /// faces of the bounding box in the -x, -y, -z world axis directions.
    pub fn minimums(&self) -> Vec3 {
        self.minimums
    }
    /// Returns the distance from the origin of the mesh to the positive extents of the bounding
    /// box in world space, aligned with the world axes. I.e.: distance from the mesh origin to the
    /// faces of the bounding box in the +x, +y, +z world axis directions.
    pub fn maximums(&self) -> Vec3 {
        self.maximums
    }
    /// Returns the vertices of the bounding box in world space, given the current mesh transform.
    pub fn vertices(&self, transform: GlobalTransform) -> [Vec3; 8] {
        let vertices_mesh_space = self.vertices_mesh_space();
        [
            transform.translation + vertices_mesh_space[0],
            transform.translation + vertices_mesh_space[1],
            transform.translation + vertices_mesh_space[2],
            transform.translation + vertices_mesh_space[3],
            transform.translation + vertices_mesh_space[4],
            transform.translation + vertices_mesh_space[5],
            transform.translation + vertices_mesh_space[6],
            transform.translation + vertices_mesh_space[7],
        ]
    }
    pub fn vertices_mesh_space(&self) -> [Vec3; 8] {
        /*
              (2)-----(3)               Y
               | \     | \              |
               |  (1)-----(0) MAX       o---X
               |   |   |   |             \
          MIN (6)--|--(7)  |              Z
                 \ |     \ |
                  (5)-----(4)
        */
        [
            Vec3::new(self.maximums.x, self.maximums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.maximums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.maximums.y, self.minimums.z),
            Vec3::new(self.maximums.x, self.maximums.y, self.minimums.z),
            Vec3::new(self.maximums.x, self.minimums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.minimums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.minimums.y, self.minimums.z),
            Vec3::new(self.maximums.x, self.minimums.y, self.minimums.z),
        ]
    }
    pub fn from_extents(minimums: Vec3, maximums: Vec3) -> Self {
        Aabb { minimums, maximums }
    }
    /// Given a set of points, fit an axis oriented bounding box to the vertices by finding the
    /// extents of the mesh.
    pub fn compute_aabb(vertices: &[Vec3]) -> Aabb {
        let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        for vertex in vertices.iter() {
            maximums = vertex.max(maximums);
            minimums = vertex.min(minimums);
        }
        Aabb { minimums, maximums }
    }
}

impl BoundingVolume for Aabb {
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self {
        let transform_matrix = Transform {
            translation: Vec3::ZERO,
            rotation: transform.rotation,
            scale: transform.scale,
        }
        .compute_matrix();
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding box generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float32x3(positions) => positions
                    .iter()
                    .map(|coordinates| transform_matrix.transform_point3(Vec3::from(*coordinates)))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        Self::compute_aabb(&vertices)
    }

    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh {
        let mut mesh = Mesh::from(self);
        let inverse_transform = Transform::from_matrix(
            Mat4::from_scale_rotation_translation(transform.scale, transform.rotation, Vec3::ZERO)
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
        for vertex in self.vertices(*bound_vol_position).iter() {
            if normal.dot(*vertex) + -normal.dot(point) < 0.0 {
                // if any point is on the inside of the plane, we can end early.
                return false;
            }
        }
        true
    }
}
