// Rust translation of ChFi2d_AnaFilletAlgo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFi2d_AnaFilletAlgo`: the analytical fillet of two neighbours that are segments or
//! arcs of circle.
mod arc_arc;
mod cases;
mod perform;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::proj_lib::project_point_on_plane;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Plane3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire};

/// The class name in the failures.
const CLASS: &str = "ChFi2d_AnaFilletAlgo";

/// One neighbour of the fillet in the parameters of the plane: a segment or an arc of
/// circle from `p1` to `p2` (`x11, y11, x12, y12` or `x21, ...`), and for an arc its
/// center, radius and direction of travel (clockwise).
#[derive(Clone, Copy, Debug, Default)]
struct Neighbour {
    segment: bool,
    p1: (f64, f64),
    p2: (f64, f64),
    center: (f64, f64),
    radius: f64,
    cw: bool,
}

/// `ChFi2d_AnaFilletAlgo`: the plane, the two neighbours, and the fillet with the
/// shrunk neighbours.
///
/// ```
/// use rustycad_tkfillet::chfi2d::ChFi2dAnaFilletAlgo;
/// let mut algo = ChFi2dAnaFilletAlgo::new();
/// assert!(!algo.perform(1.0)?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct ChFi2dAnaFilletAlgo {
    plane: Plane3,
    e1: Edge,
    n1: Neighbour,
    e2: Edge,
    n2: Neighbour,
    fillet: Edge,
    shrinke1: Edge,
    shrinke2: Edge,
}

impl ChFi2dAnaFilletAlgo {
    /// `ChFi2d_AnaFilletAlgo()`: use `init_wire` or `init_edges` next.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFi2d_AnaFilletAlgo(theWire, thePlane)`.
    pub fn with_wire(wire: &Wire, plane: &Plane3) -> Result<Self, Failure> {
        let mut algo = Self {
            plane: *plane,
            ..Self::default()
        };
        algo.init_wire(wire, plane)?;
        Ok(algo)
    }

    /// `ChFi2d_AnaFilletAlgo(theEdge1, theEdge2, thePlane)`.
    pub fn with_edges(edge1: &Edge, edge2: &Edge, plane: &Plane3) -> Result<Self, Failure> {
        let mut algo = Self {
            plane: *plane,
            ..Self::default()
        };
        algo.init_edges(edge1, edge2, plane)?;
        Ok(algo)
    }

    /// `Init(theWire, thePlane)`: the edges of the wire fill the neighbours that are
    /// still null (the previous ones are kept, as natively); Standard_TypeMismatch unless
    /// both are segments or arcs of circle.
    pub fn init_wire(&mut self, wire: &Wire, plane: &Plane3) -> Result<(), Failure> {
        self.plane = *plane;
        let mut itr = ShapeIterator::new(wire, true, true);
        while itr.more() {
            if self.e1.is_null() {
                self.e1 = topods::edge(itr.value()?)?;
            } else if self.e2.is_null() {
                self.e2 = topods::edge(itr.value()?)?;
            }
            itr.next();
        }
        if self.e1.is_null() || self.e2.is_null() {
            return Err(type_mismatch(
                "The algorithm expects a wire consisting of two linear or circular edges.",
            ));
        }
        // Left neighbour, then right neighbour.
        self.n1 = neighbour(&self.e1, plane)?;
        self.n2 = neighbour(&self.e2, plane)?;
        Ok(())
    }

    /// `Init(theEdge1, theEdge2, thePlane)`: the wire of the two edges, reversed as
    /// needed to meet at their common point; Standard_Failure without a common point.
    pub fn init_edges(
        &mut self,
        edge1: &Edge,
        edge2: &Edge,
        plane: &Plane3,
    ) -> Result<(), Failure> {
        let ([p11, p12], [p21, p22]) = (end_points(edge1)?, end_points(edge2)?);
        let pcommon = if is_equal(p11, p21) || is_equal(p11, p22) {
            p11
        } else if is_equal(p12, p21) || is_equal(p12, p22) {
            p12
        } else {
            return Err(failure("The edges have no common point."));
        };
        // Reverse the edges in case of need (to construct a wire).
        let reversed1 = is_equal(pcommon, p11);
        let reversed2 = !reversed1 && is_equal(pcommon, p22);
        let mut mk_wire = BRepBuilderAPIMakeWire::new();
        mk_wire.add_edge(&oriented(edge1, reversed1))?;
        mk_wire.add_edge(&oriented(edge2, reversed2))?;
        if !mk_wire.is_done() {
            return Err(failure("Can't make a wire."));
        }
        let wire = mk_wire.wire()?;
        self.init_wire(&wire, plane)
    }

    /// `Result(e1, e2)`: the fillet, `e1` and `e2` getting the shrunk neighbours.
    pub fn result(&self, e1: &mut Edge, e2: &mut Edge) -> Edge {
        *e1 = self.shrinke1.clone();
        *e2 = self.shrinke2.clone();
        self.fillet.clone()
    }
}

/// The neighbour of a segment or an arc of circle; Standard_TypeMismatch for another
/// curve, Standard_Failure for an infinite edge.
fn neighbour(edge: &Edge, plane: &Plane3) -> Result<Neighbour, Failure> {
    let ac = BRepAdaptorCurve::with_edge(edge)?;
    let kind = ac.get_type()?;
    if kind != CurveType::Line && kind != CurveType::Circle {
        return Err(type_mismatch("A segment or an arc of circle is expected."));
    }
    let [p1, p2] = end_points(edge)?;
    let (p1, p2) = (
        project_point_on_plane(plane, p1),
        project_point_on_plane(plane, p2),
    );
    let mut n = Neighbour {
        segment: true,
        p1: (p1.x(), p1.y()),
        p2: (p2.x(), p2.y()),
        ..Neighbour::default()
    };
    if kind == CurveType::Circle {
        let c = ac.circle()?;
        let loc = project_point_on_plane(plane, c.location());
        (n.segment, n.center, n.radius, n.cw) =
            (false, (loc.x(), loc.y()), c.radius(), is_cw(&ac)?);
    }
    Ok(n)
}

/// The points of the vertices of an edge (with its orientation); Standard_Failure for
/// an infinite edge.
fn end_points(edge: &Edge) -> Result<[Point3; 2], Failure> {
    let (v1, v2) = topexp::vertices(edge, true);
    if v1.is_null() || v2.is_null() {
        return Err(failure("An infinite edge."));
    }
    Ok([BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?])
}

/// The edge, reversed when asked.
fn oriented(edge: &Edge, reversed: bool) -> Edge {
    if reversed {
        Edge::unchecked(edge.reversed())
    } else {
        edge.clone()
    }
}

/// `isCW(AC)`: whether the arc runs clockwise around the axis of its circle (the angle
/// to its middle point beyond the angle to its end).
fn is_cw(ac: &BRepAdaptorCurve) -> Result<bool, Failure> {
    let (f, l) = (ac.first_parameter()?, ac.last_parameter()?);
    let curve = ac.transformed_curve().geom_curve();
    let circle = curve
        .get()
        .ok_or_else(|| failure("isCW: null curve; expected a circle"))?;
    let (start, end) = (ac.value(f)?, ac.value(l)?);
    let circ = ac.circle()?;
    let (center, normal) = (circ.location(), Vector3::from(circ.position().direction()));
    // Get point on circle at half angle
    let m = circle.value(0.5 * (f + l))?;
    let startv = Vector3::between(center, start);
    let middlea = positive_angle(startv.angle_with_ref(Vector3::between(center, m), normal)?);
    let enda = positive_angle(startv.angle_with_ref(Vector3::between(center, end), normal)?);
    Ok(middlea > enda)
}

/// The angle brought into [0, 2 pi) by adding turns.
fn positive_angle(mut angle: f64) -> f64 {
    while angle < 0.0 {
        angle += 2.0 * std::f64::consts::PI;
    }
    angle
}

/// `IsEqual(p1, p2)`: the square distance below Precision::SquareConfusion.
fn is_equal(p1: Point3, p2: Point3) -> bool {
    p1.square_distance(&p2) < rustycad_tkernel::precision::square_confusion()
}

fn type_mismatch(message: &str) -> Failure {
    Failure::with_message("Standard_TypeMismatch", &format!("{CLASS}: {message}"))
}

fn failure(message: &str) -> Failure {
    Failure::with_message("Standard_Failure", &format!("{CLASS}: {message}"))
}
