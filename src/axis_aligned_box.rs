use crate::IsBoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;

/// Defines
#[derive(Debug, Clone, Default)]
pub struct AxisAlignedBB {
    minimums: Vec3,
    maximums: Vec3,
}
impl AxisAlignedBB {
    pub fn minimums(&self) -> Vec3 {
        self.minimums
    }
    pub fn maximums(&self) -> Vec3 {
        self.maximums
    }
    pub fn vertices(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.maximums.x, self.maximums.y, self.maximums.z),
            Vec3::new(self.maximums.x, self.maximums.y, self.minimums.z),
            Vec3::new(self.maximums.x, self.minimums.y, self.maximums.z),
            Vec3::new(self.maximums.x, self.minimums.y, self.minimums.z),
            Vec3::new(self.minimums.x, self.maximums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.maximums.y, self.minimums.z),
            Vec3::new(self.minimums.x, self.minimums.y, self.maximums.z),
            Vec3::new(self.minimums.x, self.minimums.y, self.minimums.z),
        ]
    }
    pub fn from_extents(minimums: Vec3, maximums: Vec3) -> Self {
        AxisAlignedBB { minimums, maximums }
    }
    /// Given a set of points, fit an axis oriented bounding box to the mesh by finding the extents
    /// of the mesh.
    pub fn compute_aabb(vertices: &Vec<Vec3>) -> AxisAlignedBB {
        let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        for vertex in vertices.iter() {
            maximums = vertex.max(maximums);
            minimums = vertex.min(minimums);
        }
        AxisAlignedBB { minimums, maximums }
    }
}

impl IsBoundingVolume for AxisAlignedBB {
    fn new(mesh: &Mesh, transform: &GlobalTransform) -> Self {
        let transform_matrix = Transform {
            translation: Vec3::zero(),
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
                VertexAttributeValues::Float3(positions) => positions
                    .iter()
                    .map(|coordinates| transform_matrix.transform_point3(Vec3::from(*coordinates)))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        Self::compute_aabb(&vertices)
    }

    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh {
        let mut mesh = Mesh::from(shape::Box {
            max_x: self.maximums.x,
            max_y: self.maximums.y,
            max_z: self.maximums.z,
            min_x: self.minimums.x,
            min_y: self.minimums.y,
            min_z: self.minimums.z,
        });
        let inverse_transform = Transform::from_matrix(
            Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                Vec3::zero(),
            )
            .inverse(),
        );
        match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match vertex_values {
                VertexAttributeValues::Float3(ref mut positions) => {
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

    fn update_on_mesh_change(&self, mesh: &Mesh, transform: &GlobalTransform) -> Self {
        Self::new(mesh, transform)
    }

    fn update_on_transform_change(&self, mesh: &Mesh, transform: &GlobalTransform) -> Self {
        Self::new(mesh, transform)
    }
}
