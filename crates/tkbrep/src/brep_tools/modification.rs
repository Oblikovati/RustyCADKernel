// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_Modification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Modification`: the geometric modification that `BRepTools_Modifier`
//! applies to a shape.
use crate::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// The output parameters of `NewSurface(F, S, L, Tol, RevWires, RevFace)`. A
/// modification writes them whether or not it returns true, as natively.
#[derive(Clone, Default)]
pub struct NewSurface {
    /// `S`: the new surface.
    pub surface: Handle<dyn GeomSurface>,
    /// `L`: its location.
    pub location: Location,
    /// `Tol`: the tolerance of the new face.
    pub tolerance: f64,
    /// `RevWires`: the wires of the face must be reversed.
    pub rev_wires: bool,
    /// `RevFace`: the face must be reversed.
    pub rev_face: bool,
}

/// The output parameters of `NewCurve(E, C, L, Tol)`.
#[derive(Clone, Default)]
pub struct NewCurve {
    /// `C`: the new 3D curve (null for an edge without one).
    pub curve: Handle<dyn GeomCurve>,
    /// `L`: its location.
    pub location: Location,
    /// `Tol`: the tolerance of the new edge.
    pub tolerance: f64,
}

/// `BRepTools_Modification`: how each face, edge and vertex of a shape is modified.
///
/// The pure virtual members are required methods; `NewTriangulation`, `NewPolygon` and
/// `NewPolygonOnTriangulation` are provided with the native bodies (no new mesh). The
/// output parameters are `&mut` arguments, which a member may change even when it
/// returns false, as natively. Members take `&self`: a modification is shared through a
/// handle and keeps its state behind locks.
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, CopyModification};
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkernel::standard::Handle;
/// let modification = CopyModification::new(true, false);
/// let mut mesh = Handle::default();
/// assert!(modification.new_triangulation(&Face::default(), &mut mesh).is_err());
/// ```
pub trait BRepToolsModification: Transient {
    /// `NewSurface(F, S, L, Tol, RevWires, RevFace)`: true when the face gets a new
    /// surface, given in `result`.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure>;

    /// `NewTriangulation(F, T)`: true when the face gets a new triangulation; the base
    /// class returns false.
    fn new_triangulation(
        &self,
        _face: &Face,
        _triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    /// `NewCurve(E, C, L, Tol)`: true when the edge gets a new 3D curve (possibly null).
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure>;

    /// `NewPolygon(E, P)`: true when the edge gets a new 3D polygon; the base class
    /// returns false.
    fn new_polygon(&self, _edge: &Edge, _polygon: &mut Handle<Polygon3D>) -> Result<bool, Failure> {
        Ok(false)
    }

    /// `NewPolygonOnTriangulation(E, F, P)`: true when the edge gets a new polygon on
    /// the triangulation of the face; the base class returns false.
    fn new_polygon_on_triangulation(
        &self,
        _edge: &Edge,
        _face: &Face,
        _polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    /// `NewPoint(V, P, Tol)`: true when the vertex gets a new point.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure>;

    /// `NewCurve2d(E, F, NewE, NewF, C, Tol)`: true when the edge gets a new p-curve on
    /// the face; `new_edge` and `new_face` are the shapes being built.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure>;

    /// `NewParameter(V, E, P, Tol)`: true when the vertex gets a new parameter on the
    /// edge.
    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure>;

    /// `Continuity(E, F1, F2, NewE, NewF1, NewF2)`: the continuity of the new edge
    /// between the new faces.
    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        new_edge: &Edge,
        new_face1: &Face,
        new_face2: &Face,
    ) -> Result<Continuity, Failure>;
}
impl_transient_interface!(
    dyn BRepToolsModification,
    "BRepTools_Modification",
    StandardTransient
);
