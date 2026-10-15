// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Offset.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_Offset`: the offset face of a face, the pipe face of an edge between
//! two rails, the sphere face of a vertex, the tube face of a free edge.
mod curve3d;
mod face;
mod pipe;
mod vertex;

use super::BRepOffsetStatus;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::{JoinType, Shape as Continuity};
use rustycad_tkmath::toploc::Location;

/// The images already built of the edges and vertices of a face (`Created`).
pub type CreatedShapes = DataMap<Shape, Shape, ShapeMapHasher>;

/// The approximation options of the pipe and sphere faces: `Polynomial`, `Tol` and
/// `Conti`, natively defaulted to `false`, `1e-4` and `GeomAbs_C1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PipeOptions {
    /// Whether the surface is approximated by a polynomial one.
    pub polynomial: bool,
    /// The approximation tolerance.
    pub tol: f64,
    /// The required continuity.
    pub conti: Continuity,
}

impl Default for PipeOptions {
    fn default() -> Self {
        Self {
            polynomial: false,
            tol: 1.0e-4,
            conti: Continuity::C1,
        }
    }
}

/// `BRepOffset_Offset`: the offset face of a shape (face, edge between rails, vertex,
/// free edge) with the map of its sub-shapes to their images;
/// `BRepOffsetOffset::from_face(&face, 2.0, true, JoinType::Arc)?.face()`.
#[derive(Clone, Debug)]
pub struct BRepOffsetOffset {
    my_shape: Shape,
    my_status: BRepOffsetStatus,
    my_face: Face,
}

impl Default for BRepOffsetOffset {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepOffsetOffset {
    /// `BRepOffset_Offset()`: no shape and no face. The native status is left
    /// uninitialized; it is Good here. `BRepOffsetOffset::new()`.
    pub fn new() -> Self {
        Self {
            my_shape: Shape::default(),
            my_status: BRepOffsetStatus::Good,
            my_face: Face::default(),
        }
    }

    /// `BRepOffset_Offset(Face, Offset, OffsetOutside, JoinType)`;
    /// `BRepOffsetOffset::from_face(&face, 2.0, true, JoinType::Arc)?`.
    pub fn from_face(
        face: &Face,
        offset: f64,
        offset_outside: bool,
        join_type: JoinType,
    ) -> Result<Self, Failure> {
        let mut o = Self::new();
        o.init_face(face, offset, offset_outside, join_type)?;
        Ok(o)
    }

    /// `BRepOffset_Offset(Face, Offset, Created, OffsetOutside, JoinType)`;
    /// `BRepOffsetOffset::from_face_created(&face, 2.0, &created, true, JoinType::Arc)?`.
    pub fn from_face_created(
        face: &Face,
        offset: f64,
        created: &CreatedShapes,
        offset_outside: bool,
        join_type: JoinType,
    ) -> Result<Self, Failure> {
        let mut o = Self::new();
        o.init_face_created(face, offset, created, offset_outside, join_type)?;
        Ok(o)
    }

    /// `BRepOffset_Offset(Path, Edge1, Edge2, Offset, Polynomial, Tol, Conti)`;
    /// `BRepOffsetOffset::from_edges(&path, &e1, &e2, 1.0, PipeOptions::default())?`.
    pub fn from_edges(
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        options: PipeOptions,
    ) -> Result<Self, Failure> {
        let mut o = Self::new();
        o.init_edges(path, edge1, edge2, offset, options)?;
        Ok(o)
    }

    /// `BRepOffset_Offset(Path, Edge1, Edge2, Offset, FirstEdge, LastEdge, Polynomial,
    /// Tol, Conti)`; `BRepOffsetOffset::from_edges_with_connectors(&path, &e1, &e2, 1.0,
    /// &first, &last, PipeOptions::default())?`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_edges_with_connectors(
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        first_edge: &Edge,
        last_edge: &Edge,
        options: PipeOptions,
    ) -> Result<Self, Failure> {
        let mut o = Self::new();
        o.init_edges_with_connectors(path, edge1, edge2, offset, first_edge, last_edge, options)?;
        Ok(o)
    }

    /// `BRepOffset_Offset(Vertex, LEdge, Offset, Polynomial, Tol, Conti)`;
    /// `BRepOffsetOffset::from_vertex(&v, &edges, 1.0, PipeOptions::default())?`.
    pub fn from_vertex(
        vertex: &Vertex,
        l_edge: &List<Shape>,
        offset: f64,
        options: PipeOptions,
    ) -> Result<Self, Failure> {
        let mut o = Self::new();
        o.init_vertex(vertex, l_edge, offset, options)?;
        Ok(o)
    }

    /// `Init(Face, Offset, OffsetOutside, JoinType)`: `init_face_created` with no
    /// created images; `o.init_face(&face, 2.0, true, JoinType::Arc)?`.
    pub fn init_face(
        &mut self,
        face: &Face,
        offset: f64,
        offset_outside: bool,
        join_type: JoinType,
    ) -> Result<(), Failure> {
        self.init_face_created(
            face,
            offset,
            &CreatedShapes::default(),
            offset_outside,
            join_type,
        )
    }

    /// `Init(Path, Edge1, Edge2, Offset, Polynomial, Tol, Conti)`: no connectors;
    /// `o.init_edges(&path, &e1, &e2, 1.0, PipeOptions::default())?`.
    pub fn init_edges(
        &mut self,
        path: &Edge,
        edge1: &Edge,
        edge2: &Edge,
        offset: f64,
        options: PipeOptions,
    ) -> Result<(), Failure> {
        let none = Edge::default();
        self.init_edges_with_connectors(path, edge1, edge2, offset, &none, &none, options)
    }

    /// `InitialShape()`: the shape of the last `Init`; `o.initial_shape()`.
    pub fn initial_shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `Face()`: the offset face; `o.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `Status()`: the status of the offset surface of a face, Good otherwise;
    /// `o.status()`.
    pub fn status(&self) -> BRepOffsetStatus {
        self.my_status
    }

    /// `Generated(Shape)`: the image of an edge of the face (the edges of both faces
    /// paired in explorer order), or the connector edge at a vertex of the path; null
    /// otherwise; `o.generated(&edge)`.
    pub fn generated(&self, shape: &Shape) -> Shape {
        self.generated_shape(shape).unwrap_or_default()
    }

    fn generated_shape(&self, shape: &Shape) -> Result<Shape, Failure> {
        if self.my_shape.is_null() {
            return Ok(Shape::default());
        }
        match self.my_shape.shape_type()? {
            ShapeEnum::Face => {
                let mut exp = Explorer::new(
                    &self.my_shape.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )?;
                let mut expo = Explorer::new(
                    &self.my_face.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )?;
                while exp.more() && expo.more() {
                    if shape.is_same(exp.current()) {
                        if self.my_shape.orientation() == Orientation::Reversed {
                            return Ok(expo.current().reversed());
                        }
                        return Ok(expo.current().clone());
                    }
                    exp.next()?;
                    expo.next()?;
                }
                Ok(Shape::default())
            }
            ShapeEnum::Edge => {
                let (_, v2) = topexp::vertices(&topods::edge(&self.my_shape)?, false);
                let expf = Explorer::new(
                    &self.my_face.oriented(Orientation::Forward),
                    ShapeEnum::Wire,
                    ShapeEnum::Shape,
                )?;
                let wire = expf.current().clone();
                let mut expo = Explorer::new(
                    &wire.oriented(Orientation::Forward),
                    ShapeEnum::Edge,
                    ShapeEnum::Shape,
                )?;
                expo.next()?;
                expo.next()?;
                if !v2.is_same(shape) {
                    expo.next()?;
                }
                let mut a_shape = if wire.orientation() == Orientation::Reversed {
                    expo.current().reversed()
                } else {
                    expo.current().clone()
                };
                if self.my_face.orientation() == Orientation::Reversed {
                    a_shape.reverse();
                }
                Ok(a_shape)
            }
            _ => Ok(Shape::default()),
        }
    }
}

/// `UpdateEdge(E, C, L, Tol)`: the 3d curve of the edge, the basis of a trimmed one.
pub(super) fn update_edge_curve(
    e: &Edge,
    c: &Handle<dyn GeomCurve>,
    l: &Location,
    tol: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if c.is_instance_name("Geom_TrimmedCurve") {
        let bc = c.clone().cast::<GeomTrimmedCurve>();
        let basis = bc.get().map(|t| t.basis_curve()).unwrap_or_default();
        return builder.update_edge_located_curve(e, &basis, l, tol);
    }
    builder.update_edge_located_curve(e, c, l, tol)
}

/// The basis of a trimmed 2d curve, the curve otherwise.
fn untrimmed_2d(c: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    if c.is_instance_name("Geom2d_TrimmedCurve") {
        let bc = c.clone().cast::<Geom2dTrimmedCurve>();
        return bc.get().map(|t| t.basis_curve()).unwrap_or_default();
    }
    c.clone()
}

/// `UpdateEdge(E, C, F, Tol)`: the p-curve of the edge on the face, the basis of a
/// trimmed one.
pub(super) fn update_edge_pcurve(
    e: &Edge,
    c: &Handle<dyn Geom2dCurve>,
    f: &Face,
    tol: f64,
) -> Result<(), Failure> {
    BRepBuilder::default().update_edge_pcurve_on_face(e, &untrimmed_2d(c), f, tol)
}

/// `UpdateEdge(E, C1, C2, F, Tol)`: the p-curves of the closed edge on the face, the
/// bases of trimmed ones.
pub(super) fn update_edge_pcurves(
    e: &Edge,
    c1: &Handle<dyn Geom2dCurve>,
    c2: &Handle<dyn Geom2dCurve>,
    f: &Face,
    tol: f64,
) -> Result<(), Failure> {
    let (nc1, nc2) = (untrimmed_2d(c1), untrimmed_2d(c2));
    BRepBuilder::default().update_edge_pcurves_on_face(e, (&nc1, &nc2), f, tol)
}

/// Standard_ConstructionError of a failed pipe surface.
pub(super) fn pipe_failure() -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        "GeomFill_Pipe : Cannot make a surface",
    )
}
