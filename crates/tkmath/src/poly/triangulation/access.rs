// Rust translation of the inline element accessors of FoundationClasses/TKMath/Poly/Poly_Triangulation.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{Triangle, Triangulation, in_member};
use crate::{
    gp::{Direction3, Point2, Point3},
    poly::errors::zero_based,
};
use rustycad_tkernel::{ncollection::Vec3, standard::Failure};
impl Triangulation {
    /// True when both nodes and triangles exist; `triangulation.has_geometry()`.
    pub fn has_geometry(&self) -> bool {
        let fields = self.fields.read();
        !fields.nodes.is_empty() && !fields.triangles.is_empty()
    }
    /// Number of nodes; `triangulation.nb_nodes()`.
    pub fn nb_nodes(&self) -> i32 {
        self.fields.read().nodes.len() as i32
    }
    /// Number of triangles; `triangulation.nb_triangles()`.
    pub fn nb_triangles(&self) -> i32 {
        self.fields.read().triangles.len() as i32
    }
    /// True when UV nodes are allocated; `triangulation.has_uv_nodes()`.
    pub fn has_uv_nodes(&self) -> bool {
        !self.fields.read().uv_nodes.is_empty()
    }
    /// True when normals are allocated; `triangulation.has_normals()`.
    pub fn has_normals(&self) -> bool {
        !self.fields.read().normals.is_empty()
    }
    /// Returns the node at a one-based index; `triangulation.node(1)?`.
    pub fn node(&self, index: i32) -> Result<Point3, Failure> {
        let fields = self.fields.read();
        let member = || format!("Poly_Triangulation::Node({index})");
        fields
            .nodes
            .value(zero_based(index))
            .map_err(|e| in_member(&member(), e))
    }
    /// Sets the node at a one-based index; `triangulation.set_node(1, point)?`.
    pub fn set_node(&self, index: i32, point: Point3) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let member = || format!("Poly_Triangulation::SetNode({index})");
        fields
            .nodes
            .set_value(zero_based(index), point)
            .map_err(|e| in_member(&member(), e))
    }
    /// Returns the UV node at a one-based index; `triangulation.uv_node(1)?`.
    pub fn uv_node(&self, index: i32) -> Result<Point2, Failure> {
        let fields = self.fields.read();
        let member = || format!("Poly_Triangulation::UVNode({index})");
        fields
            .uv_nodes
            .value(zero_based(index))
            .map_err(|e| in_member(&member(), e))
    }
    /// Sets the UV node at a one-based index; `triangulation.set_uv_node(1, uv)?`.
    pub fn set_uv_node(&self, index: i32, point: Point2) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let member = || format!("Poly_Triangulation::SetUVNode({index})");
        fields
            .uv_nodes
            .set_value(zero_based(index), point)
            .map_err(|e| in_member(&member(), e))
    }
    /// Returns the triangle at an index of the triangle array; `triangulation.triangle(1)?`.
    pub fn triangle(&self, index: i32) -> Result<Triangle, Failure> {
        let fields = self.fields.read();
        let member = || format!("Poly_Triangulation::Triangle({index})");
        fields
            .triangles
            .value(index)
            .copied()
            .map_err(|e| e.into_failure(&member()))
    }
    /// Sets the triangle at an index of the triangle array; `triangulation.set_triangle(1, t)?`.
    pub fn set_triangle(&self, index: i32, triangle: Triangle) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let member = || format!("Poly_Triangulation::SetTriangle({index})");
        fields
            .triangles
            .set_value(index, triangle)
            .map_err(|e| e.into_failure(&member()))
    }
    /// Returns the stored single-precision normal; `triangulation.normal_vec3(1)?`.
    pub fn normal_vec3(&self, index: i32) -> Result<Vec3<f32>, Failure> {
        let fields = self.fields.read();
        let member = || format!("Poly_Triangulation::Normal({index})");
        fields
            .normals
            .value(index.saturating_sub(1))
            .copied()
            .map_err(|e| e.into_failure(&member()))
    }
    /// Returns the normal widened to a `gp_Dir`, which fails for a null vector;
    /// `triangulation.normal(1)?`.
    pub fn normal(&self, index: i32) -> Result<Direction3, Failure> {
        let normal = self.normal_vec3(index)?;
        Direction3::new(
            f64::from(normal.x()),
            f64::from(normal.y()),
            f64::from(normal.z()),
        )
    }
    /// Stores a single-precision normal; `triangulation.set_normal_vec3(1, Vec3::new(0.0, 0.0, 1.0))?`.
    pub fn set_normal_vec3(&self, index: i32, normal: Vec3<f32>) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        let member = || format!("Poly_Triangulation::SetNormal({index})");
        fields
            .normals
            .set_value(index.saturating_sub(1), normal)
            .map_err(|e| e.into_failure(&member()))
    }
    /// Stores a direction narrowed to single precision; `triangulation.set_normal(1, Direction3::Z)?`.
    pub fn set_normal(&self, index: i32, normal: Direction3) -> Result<(), Failure> {
        let narrowed = Vec3::new(normal.x() as f32, normal.y() as f32, normal.z() as f32);
        self.set_normal_vec3(index, narrowed)
    }
}
