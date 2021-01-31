use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;
use std::f32::consts::PI;

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
}

impl From<&Mesh> for AxisAlignedBoundingBox {
    fn from(mesh: &Mesh) -> Self {
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding box generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float3(positions) => positions
                    .iter()
                    .map(|coordinates| Vec3::from(*coordinates))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        compute_aabb(&vertices, None)
    }
}

/// Given a set of points, apply the transform (if provided) and fit an axis oriented bounding box
/// to the mesh by finding the extents of the mesh in its transformed position.
fn compute_aabb(vertices: &Vec<Vec3>, transform: Option<Mat4>) -> AxisAlignedBoundingBox {
    let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    match transform {
        Some(transform) => {
            for vertex in vertices.iter() {
                maximums = maximums.max(transform.transform_point3(*vertex));
                minimums = maximums.min(transform.transform_point3(*vertex));
            }
        }
        None => { 
            for vertex in vertices.iter() {
                maximums = vertex.max(maximums);
                minimums = vertex.min(minimums);
            }
        }
    }
    let dimensions = maximums - minimums;
    let origin = minimums;
    AxisAlignedBoundingBox { origin, dimensions }
}

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug, Clone)]
pub struct OrientedBoundingBox {
    origin: Vec3,
    dimensions: Vec3,
    orientation: Mat4,
}

impl OrientedBoundingBox {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn dimensions(&self) -> Vec3 {
        self.dimensions
    }
    pub fn orientation(&self) -> Mat4 {
        self.orientation
    }
}

impl From<&Mesh> for OrientedBoundingBox {
    fn from(mesh: &Mesh) -> Self {
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for oriented bounding box generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float3(positions) => positions
                    .iter()
                    .map(|coordinates| Vec3::from(*coordinates))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        let mut orientation = Mat4::default();
        let mut volume = f32::MAX;
        for x_angle in (0..180).step_by(10) {
            let new_orientation = Mat4::from_rotation_x(x_angle as f32*2.0*PI/360.0);
            let dims = compute_aabb(&vertices, Some(new_orientation)).dimensions;
            let new_volume = dims.x * dims.y * dims.z;
            if new_volume < volume {
                volume = new_volume;
                orientation = new_orientation;
            }
        }
        let orientation_x = orientation;
        for y_angle in (0..180).step_by(10) {
            let new_orientation = orientation_x*Mat4::from_rotation_y(y_angle as f32*2.0*PI/360.0) ;
            let dims = compute_aabb(&vertices, Some(new_orientation)).dimensions;
            let new_volume = dims.x * dims.y * dims.z;
            if new_volume < volume {
                volume = new_volume;
                orientation = new_orientation;
            }
        }
        let orientation_y = orientation;
        for z_angle in (0..180).step_by(10) {
            let new_orientation = orientation_y*Mat4::from_rotation_z(z_angle as f32*2.0*PI/360.0);
            let dims = compute_aabb(&vertices, Some(new_orientation)).dimensions;
            let new_volume = dims.x * dims.y * dims.z;
            if new_volume < volume {
                volume = new_volume;
                orientation = new_orientation;
            }
        }
        let aabb = compute_aabb(&vertices, Some(orientation));
        OrientedBoundingBox{
            origin: aabb.origin,
            dimensions: aabb.dimensions,
            orientation: orientation.inverse(),
        }
    }
}
