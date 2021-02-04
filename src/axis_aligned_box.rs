use crate::IsBoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
    transform,
};
use core::panic;

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug, Clone)]
pub struct AxisAlignedBoundingBox {
    origin: Vec3,
    dimensions: Vec3,
}
impl AxisAlignedBoundingBox {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn dimensions(&self) -> Vec3 {
        self.dimensions
    }
    /// Given a set of points, fit an axis oriented bounding box to the mesh by finding the extents
    /// of the mesh.
    fn compute_aabb(vertices: &Vec<Vec3>) -> AxisAlignedBoundingBox {
        let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        for vertex in vertices.iter() {
            maximums = vertex.max(maximums);
            minimums = vertex.min(minimums);
        }
        let dimensions = maximums;
        let origin = minimums;
        AxisAlignedBoundingBox { origin, dimensions }
    }
}

impl IsBoundingVolume for AxisAlignedBoundingBox {
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
        println!("AABB created");
        Self::compute_aabb(&vertices)
    }

    fn new_debug_mesh(&self, transform: &GlobalTransform) -> Mesh {
        let mut mesh = Mesh::from(shape::Box {
            max_x: self.dimensions.x,
            max_y: self.dimensions.y,
            max_z: self.dimensions.z,
            min_x: self.origin.x,
            min_y: self.origin.y,
            min_z: self.origin.z,
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
        println!("new aabb debug mesh");
        mesh
    }
}
