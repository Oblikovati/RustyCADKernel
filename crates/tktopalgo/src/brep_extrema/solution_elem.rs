// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_SolutionElem.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_SolutionElem`: one solution point of a distance computation with its
//! support (vertex, edge or face) and its parameters on that support.
use super::BRepExtremaSupportType;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkmath::gp::Point3;

/// `BRepExtrema_SolutionElem` (docs/topalgo-design.md section 13): the distance, the
/// point, the support kind, the support itself (only the field of its kind is set) and
/// up to two parameters (`t` on an edge, `(u, v)` in a face; 0 otherwise).
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::{BRepExtremaSolutionElem, BRepExtremaSupportType};
/// let empty = BRepExtremaSolutionElem::default();
/// assert_eq!(empty.support_kind(), BRepExtremaSupportType::IsVertex);
/// assert_eq!(empty.dist(), 0.0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepExtremaSolutionElem {
    my_dist: f64,
    my_point: Point3,
    my_sup_type: BRepExtremaSupportType,
    my_vertex: Vertex,
    my_edge: Edge,
    my_face: Face,
    my_par1: f64,
    my_par2: f64,
}

impl BRepExtremaSolutionElem {
    /// The empty solution: distance 0 at the origin, support `IsVertex`, null shapes;
    /// `BRepExtremaSolutionElem::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepExtrema_SolutionElem(theDist, thePoint, theSolType, theVertex)`: a solution
    /// on a vertex; `BRepExtremaSolutionElem::with_vertex(1.0, p, kind, &vertex)`.
    pub fn with_vertex(
        dist: f64,
        point: Point3,
        sol_type: BRepExtremaSupportType,
        vertex: &Vertex,
    ) -> Self {
        Self {
            my_dist: dist,
            my_point: point,
            my_sup_type: sol_type,
            my_vertex: vertex.clone(),
            ..Self::default()
        }
    }

    /// `BRepExtrema_SolutionElem(theDist, thePoint, theSolType, theEdge, theParam)`: a
    /// solution on an edge at `param`;
    /// `BRepExtremaSolutionElem::with_edge(1.0, p, kind, &edge, 0.5)`.
    pub fn with_edge(
        dist: f64,
        point: Point3,
        sol_type: BRepExtremaSupportType,
        edge: &Edge,
        param: f64,
    ) -> Self {
        Self {
            my_dist: dist,
            my_point: point,
            my_sup_type: sol_type,
            my_edge: edge.clone(),
            my_par1: param,
            ..Self::default()
        }
    }

    /// `BRepExtrema_SolutionElem(theDist, thePoint, theSolType, theFace, theU, theV)`: a
    /// solution inside a face at `(u, v)`;
    /// `BRepExtremaSolutionElem::with_face(1.0, p, kind, &face, 0.5, 0.25)`.
    pub fn with_face(
        dist: f64,
        point: Point3,
        sol_type: BRepExtremaSupportType,
        face: &Face,
        u: f64,
        v: f64,
    ) -> Self {
        Self {
            my_dist: dist,
            my_point: point,
            my_sup_type: sol_type,
            my_face: face.clone(),
            my_par1: u,
            my_par2: v,
            ..Self::default()
        }
    }

    /// `Dist()`: the distance of the solution; `solution.dist()`.
    pub fn dist(&self) -> f64 {
        self.my_dist
    }

    /// `Point()`: the solution point; `solution.point()`.
    pub fn point(&self) -> &Point3 {
        &self.my_point
    }

    /// `SupportKind()`: the kind of the support; `solution.support_kind()`.
    pub fn support_kind(&self) -> BRepExtremaSupportType {
        self.my_sup_type
    }

    /// `Vertex()`: the vertex support (null unless built on a vertex); `solution.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        &self.my_vertex
    }

    /// `Edge()`: the edge support (null unless built on an edge); `solution.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.my_edge
    }

    /// `Face()`: the face support (null unless built on a face); `solution.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `EdgeParameter(theParam)`: the first parameter, whatever the support;
    /// `let t = solution.edge_parameter();`.
    pub fn edge_parameter(&self) -> f64 {
        self.my_par1
    }

    /// `FaceParameter(theU, theV)`: the two parameters, whatever the support;
    /// `let (u, v) = solution.face_parameter();`.
    pub fn face_parameter(&self) -> (f64, f64) {
        (self.my_par1, self.my_par2)
    }
}
