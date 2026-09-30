// Rust translation of ModelingAlgorithms/TKShHealing/ShapeBuild/ShapeBuild_Vertex.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeBuild_Vertex`: the smallest vertex that holds two others.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `ShapeBuild_Vertex`: low-level tools to construct vertices.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkshhealing::shape_build::ShapeBuildVertex;
/// let v = ShapeBuildVertex
///     .combine_points(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0), 1.0, 1.0, 1.0)
///     .unwrap();
/// assert!(!v.is_null());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeBuildVertex;

impl ShapeBuildVertex {
    /// `CombineVertex(V1, V2, tolFactor)`: [`Self::combine_points`] on the points and
    /// tolerances of the two vertices; the native default factor is 1.0001;
    /// `ShapeBuildVertex.combine_vertices(&v1, &v2, 1.0001)?`.
    pub fn combine_vertices(
        &self,
        v1: &Vertex,
        v2: &Vertex,
        tol_factor: f64,
    ) -> Result<Vertex, Failure> {
        self.combine_points(
            BRepTool::pnt(v1)?,
            BRepTool::pnt(v2)?,
            BRepTool::vertex_tolerance(v1)?,
            BRepTool::vertex_tolerance(v2)?,
            tol_factor,
        )
    }

    /// `CombineVertex(pnt1, pnt2, tol1, tol2, tolFactor)`: a new vertex whose sphere holds
    /// both spheres `(pnt, tol)`: one of them when it holds the other, else the sphere
    /// through both far ends; its tolerance is the radius times `tol_factor`;
    /// `ShapeBuildVertex.combine_points(p1, p2, 1e-3, 1e-3, 1.0001)?`.
    pub fn combine_points(
        &self,
        pnt1: Point3,
        pnt2: Point3,
        tol1: f64,
        tol2: f64,
        tol_factor: f64,
    ) -> Result<Vertex, Failure> {
        let v = Vector3::from(pnt2.xyz() - pnt1.xyz());
        let dist = v.magnitude();
        // #47 rln 09.12.98 S4054 PRO14323 entity 2844
        let (pos, tol) = if dist + tol2 <= tol1 {
            (pnt1, tol1)
        } else if dist + tol1 <= tol2 {
            (pnt2, tol2)
        } else {
            let tol = 0.5 * (dist + tol1 + tol2);
            // szv#4:S4163:12Mar99 anti-exception
            let s = if dist > 0.0 {
                (tol2 - tol1) / dist
            } else {
                0.0
            };
            let sum = pnt1.xyz() * (1.0 - s) + pnt2.xyz() * (1.0 + s);
            (Point3::from(sum * 0.5), tol)
        };
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, pos, tol_factor * tol)?;
        Ok(vertex)
    }
}
