use crate::IsBoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;
use std::f32::consts::PI;

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug, Clone)]
pub struct OrientedBoundingBox {
    origin: Vec3,
    dimensions: Vec3,
    orientation: Quat,
}

impl OrientedBoundingBox {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn dimensions(&self) -> Vec3 {
        self.dimensions
    }
    pub fn orientation(&self) -> Quat {
        self.orientation
    }
    fn compute_obb(vertices: &Vec<Vec3>, transform: &Mat4) -> OrientedBoundingBox {
        let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        for vertex in vertices.iter() {
            maximums = maximums.max(transform.transform_point3(*vertex));
            minimums = minimums.min(transform.transform_point3(*vertex));
        }
        let dimensions = maximums;
        let origin = minimums;
        let orientation = Quat::from_rotation_mat4(transform);
        OrientedBoundingBox {
            origin,
            dimensions,
            orientation,
        }
    }

    pub fn update(
        meshes: Res<Assets<Mesh>>,
        mut query: Query<
            (&mut OrientedBoundingBox, &GlobalTransform, &Handle<Mesh>),
            Changed<Handle<Mesh>>,
        >,
    ) {
        for (mut bounding_vol, transform, handle) in query.iter_mut() {
            let mesh = meshes
                .get(handle)
                .expect("Bounding volume had bad mesh handle");
            *bounding_vol = OrientedBoundingBox::new(mesh, transform);
        }
    }
}

impl IsBoundingVolume for OrientedBoundingBox {
    fn new(mesh: &Mesh, _transform: &GlobalTransform) -> Self {
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

        let mut orientation = Mat4::from_quat(Quat::identity());
        let mut volume = f32::MAX;
        for step in 0..3 {
            // Rotate about y-axis  (turntable) until the smallest volume box is found
            let orientation_temp = orientation;
            for angle in (0..90).step_by(5) {
                let new_orientation = orientation_temp
                    * match step {
                        0 => Mat4::from_rotation_x(angle as f32 * 2.0 * PI / 360.0),
                        1 => Mat4::from_rotation_y(angle as f32 * 2.0 * PI / 360.0),
                        2 => Mat4::from_rotation_z(angle as f32 * 2.0 * PI / 360.0),
                        _ => panic!("Unreachable match arm reached!"),
                    };
                let obb = OrientedBoundingBox::compute_obb(&vertices, &new_orientation);
                let diff = obb.dimensions - obb.origin;
                let new_volume = diff.x * diff.y * diff.z;
                if new_volume < volume {
                    volume = new_volume;
                    orientation = new_orientation;
                }
            }
        }
        OrientedBoundingBox::compute_obb(&vertices, &orientation)
    }

    fn new_debug_mesh(&self, _transform: &GlobalTransform) -> Mesh {
        let mut mesh = Mesh::from(shape::Box {
            max_x: self.dimensions.x,
            max_y: self.dimensions.y,
            max_z: self.dimensions.z,
            min_x: self.origin.x,
            min_y: self.origin.y,
            min_z: self.origin.z,
        });
        let transform = Mat4::from_quat(self.orientation).inverse();
        match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match vertex_values {
                VertexAttributeValues::Float3(ref mut positions) => {
                    *positions = positions
                        .iter()
                        .map(|coordinates| {
                            transform.transform_point3(Vec3::from(*coordinates)).into()
                        })
                        .collect()
                }
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        mesh
    }
}
