use crate::aabb::AABB;
use crate::BoundingVolume;
use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, pipeline::PrimitiveTopology},
};
use core::panic;
use std::{convert::TryInto, f32::consts::PI};

/// Defines a bounding box, oriented to minimize the bounded volume. This bounding box is expensive
/// to compute, but cheap to update.
///
/// The volume of an OBB is <= to the AABB of the same mesh. It is similar to an AABB, but the
/// orientation is determined not by the world axes but with respect to the mesh itself, so the
/// bounding box definition only changes if the underlying mesh changes. The entire bounding volume
/// can simply be transformed with the current [GlobalTransform] of the bounded mesh, lazily.
///
/// This structure stores the AABB of the mesh in mesh space, with the mesh oriented to minimize
/// the volume of the bounding box. The properties are stored in mesh space to minimize rounding
/// error, and make it easy to defer recomputing the bounding volume until the mesh itself is
/// changed.
#[derive(Debug, Clone, Default)]
pub struct OBB {
    aabb: AABB,
    /// The orientation of the mesh that minimizes the AABB.
    ///
    /// ## Note
    /// This is *not* the orientation of the bounding box! You probably want the conjugate of
    /// this quaternion if that's what you need.
    mesh_orientation: Quat,
}

impl OBB {
    /// Returns an array of the 8 vertices of the bounding box in world space.
    pub fn vertices(&self, transform: GlobalTransform) -> [Vec3; 8] {
        let orient = Mat4::from_quat(self.orientation());
        let transform = transform.compute_matrix() * orient;
        let transform = GlobalTransform::from_matrix(transform);
        self.aabb
            .vertices_mesh_space()
            .iter()
            .map(|&vertex| transform * vertex)
            .collect::<Vec<Vec3>>()
            .as_slice()
            .try_into()
            .unwrap()
    }
    pub fn vertices_mesh_space(&self) -> [Vec3; 8] {
        let orient = Mat4::from_quat(self.orientation());
        let transform = GlobalTransform::from_matrix(orient);
        self.aabb.vertices(transform)
    }
    pub fn from_aabb_orientation(aabb: AABB, mesh_orientation: Quat) -> OBB {
        OBB {
            aabb,
            mesh_orientation,
        }
    }
    /// Returns the [AxisAlignedBB] of this [OrientedBB] in ***mesh space***.
    pub fn mesh_aabb(&self) -> &AABB {
        &self.aabb
    }
    /// Returns the orientation of the [OrientedBB] in ***mesh space***.
    ///
    /// ## Note
    /// This orientation tells you how to rotate the [AxisAlignedBB] that defines the [OrientedBB]
    /// so that the bounding box matches its [Mesh]s orientation.
    pub fn orientation(&self) -> Quat {
        self.mesh_orientation.conjugate()
    }
    /// Returns an [AxisAlignedBB] that contains this [OrientedBB]. In other words, this returns
    /// the AABB of this OBB.
    ///
    /// ## Y tho
    /// This is much faster than calculating the AABB of a high-poly mesh every time it moves.
    /// Because the [OrientedBB] only needs to recompute when the mesh itself changes, by taking
    /// the AABB of the OBB, and not the mesh, we only need to iterate through all mesh vertices
    /// when the mesh changes, but we still get a bounding box that is aligned to the world axes.
    /// This comes with a tradeoff - because we are finding the AABB of the OBB, the bounding box
    /// will be more conservative, and will be larger than the AABB of the mesh itself.
    pub fn outer_aabb(&self) -> AABB {
        let axis_aligned_vertices = self.aabb.vertices_mesh_space();
        let oriented_vertices: Vec<Vec3> = axis_aligned_vertices
            .iter()
            .map(|vertex| self.orientation().mul_vec3(*vertex))
            .collect();
        AABB::compute_aabb(&oriented_vertices)
    }
    /// Given a list of mesh vertices, and the orientation of this mesh, constructs an oriented
    /// bounding box.
    fn compute_obb(vertices: &[Vec3], orientation: Quat) -> OBB {
        let mut maximums = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        let mut minimums = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let transform = Mat4::from_quat(orientation);
        for vertex in vertices.iter() {
            maximums = maximums.max(transform.transform_point3(*vertex));
            minimums = minimums.min(transform.transform_point3(*vertex));
        }
        OBB {
            aabb: AABB::from_extents(minimums, maximums),
            mesh_orientation: orientation,
        }
    }
}

impl BoundingVolume for OBB {
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

        let mut orientation = Quat::IDENTITY;
        let mut volume = f32::MAX;
        // Rotate about y-axis  (turntable) until the smallest volume box is found
        let orientation_temp = orientation;
        for angle in (0..45).step_by(15) {
            let new_orientation =
                orientation_temp * Quat::from_rotation_y(angle as f32 * 2.0 * PI / 360.0);
            let temp_obb = OBB::compute_obb(&vertices, new_orientation);
            let diff = temp_obb.mesh_aabb().maximums() - temp_obb.mesh_aabb().minimums();
            let new_volume = diff.x * diff.y * diff.z;
            if new_volume < volume {
                volume = new_volume;
                orientation = new_orientation;
            }
        }
        let mut obb = OBB::compute_obb(&vertices, orientation);
        let orientation_temp = orientation;
        for angle in (0..90).step_by(15) {
            let new_orientation =
                orientation_temp * Quat::from_rotation_x(angle as f32 * 2.0 * PI / 360.0);
            let temp_obb = OBB::compute_obb(&vertices, new_orientation);
            let diff = temp_obb.mesh_aabb().maximums() - temp_obb.mesh_aabb().minimums();
            let new_volume = diff.x * diff.y * diff.z;
            if new_volume < volume {
                volume = new_volume;
                obb = temp_obb;
            }
        }
        obb
    }

    fn new_debug_mesh(&self, _transform: &GlobalTransform) -> Mesh {
        Mesh::from(self)
    }

    fn update_on_transform_change(
        &self,
        _mesh: &Mesh,
        _transform: &GlobalTransform,
    ) -> Option<Self> {
        // No-op
        None
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
