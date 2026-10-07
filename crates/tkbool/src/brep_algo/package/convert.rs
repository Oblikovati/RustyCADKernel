// Rust translation of BRepAlgo::ConvertWire and BRepAlgo::ConvertFace
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wires and faces whose edges are approximated by arcs and segments in the
//! parameter space of the face.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBoundedCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom2d_convert::ApproxArcsSegments;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

/// `MINIMAL_TOLERANCE`: the minimal tolerance of the approximation (edges may have a
/// smaller one).
const MINIMAL_TOLERANCE: f64 = 0.0001;

/// An approximated elementary curve with its sense in the wire.
struct OrientedCurve {
    curve: Handle<Geom2dTrimmedCurve>,
    is_reverse: bool,
}

impl OrientedCurve {
    /// The point at the start (or end) of the curve in the sense of the wire.
    fn point(&self, is_end: bool) -> Result<Point2, Failure> {
        let c = self.curve.get().ok_or_else(null_curve)?;
        if is_end == self.is_reverse {
            c.start_point()
        } else {
            c.end_point()
        }
    }
}

/// The elementary curves of the edges of the wire in their connection order (reversed
/// for a reversed edge), and the greatest edge tolerance (at least the minimum).
fn approximated_curves(
    the_wire: &Wire,
    the_angle_tol: f64,
    the_face: &Face,
) -> Result<(Vec<OrientedCurve>, f64), Failure> {
    let mut a_max_tol: f64 = 0.;
    let mut vec_curve: Vec<OrientedCurve> = Vec::new();
    let mut an_exp_e = WireExplorer::with_wire_on_face(the_wire, the_face)?;
    while an_exp_e.more() {
        let an_edge: Edge = an_exp_e.current().clone();
        an_exp_e.next()?;
        let a_curve = BRepAdaptorCurve2d::with_edge_on_face(&an_edge, the_face)?;
        let a_tol: f64 = BRepTool::edge_tolerance(&an_edge)?.max(MINIMAL_TOLERANCE);
        a_max_tol = a_max_tol.max(a_tol);
        let an_algo = ApproxArcsSegments::new(&a_curve, a_tol, the_angle_tol)?;
        let is_reverse: bool = an_edge.orientation() == Orientation::Reversed;
        let mut pieces: Vec<Handle<dyn Geom2dCurve>> =
            an_algo.get_result().iter().cloned().collect();
        if is_reverse {
            pieces.reverse();
        }
        for a_crv in pieces.into_iter().filter(|c| !c.is_null()) {
            vec_curve.push(OrientedCurve {
                curve: a_crv.cast(),
                is_reverse,
            });
        }
    }
    Ok((vec_curve, a_max_tol))
}

/// A vertex at the UV point of the surface with the tolerance.
fn vertex_at(a_surf: &dyn GeomSurface, p: Point2, tol: f64) -> Result<Vertex, Failure> {
    let a_pnt3d: Point3 = a_surf.value(p.x(), p.y())?;
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, a_pnt3d, tol)?;
    Ok(v)
}

/// The middle of two points.
fn middle(a: Point2, b: Point2) -> Point2 {
    Point2::from((a.xy() + b.xy()) * 0.5)
}

/// The first vertex: at the middle of the ends of the wire when they are within the
/// tolerance, else at its start.
fn first_vertex(
    vec_curve: &[OrientedCurve],
    a_surf: &dyn GeomSurface,
    a_max_tol: f64,
) -> Result<Vertex, Failure> {
    let first: Point2 = vec_curve[0].point(false)?;
    let last: Point2 = vec_curve[vec_curve.len() - 1].point(true)?;
    let a_dist: f64 = first.distance(last);
    if a_dist > a_max_tol + precision::confusion() {
        return vertex_at(a_surf, first, precision::confusion());
    }
    vertex_at(
        a_surf,
        middle(first, last),
        0.5 * a_dist + precision::confusion(),
    )
}

/// The vertex after the curve `i`: the first vertex for a closed end, else at the
/// middle of the gap to the next curve.
fn next_vertex(
    vec_curve: &[OrientedCurve],
    i: usize,
    (a_surf, a_max_tol, a_first_vertex): (&dyn GeomSurface, f64, &Vertex),
) -> Result<Vertex, Failure> {
    let end: Point2 = vec_curve[i].point(true)?;
    if i + 1 == vec_curve.len() {
        let a_dist: f64 = end.distance(vec_curve[0].point(false)?);
        if a_dist > a_max_tol + precision::confusion() {
            return vertex_at(a_surf, end, precision::confusion());
        }
        return Ok(a_first_vertex.clone());
    }
    let next: Point2 = vec_curve[i + 1].point(false)?;
    vertex_at(
        a_surf,
        middle(end, next),
        0.5 * end.distance(next) + precision::confusion(),
    )
}

/// `BRepAlgo::ConvertWire(theWire, theAngleTolerance, theFace)`: the wire of the arcs
/// and segments approximating its edges on the face within their tolerances (at least
/// 1e-4) and the angular tolerance; a null wire without edge;
/// `brep_algo::convert_wire(&wire, 0.1, &face)?`.
pub fn convert_wire(
    the_wire: &Wire,
    the_angle_tolerance: f64,
    the_face: &Face,
) -> Result<Wire, Failure> {
    let a_surf_h: Handle<dyn GeomSurface> = BRepTool::surface(the_face)?;
    let (vec_curve, a_max_tol) = approximated_curves(the_wire, the_angle_tolerance, the_face)?;
    if vec_curve.is_empty() {
        return Ok(Wire::default());
    }
    let a_surf = a_surf_h.get().ok_or_else(null_curve)?;
    let a_first_vertex = first_vertex(&vec_curve, a_surf, a_max_tol)?;
    let mut a_mk_wire = BRepBuilderAPIMakeWire::new();
    let mut a_vertex: Vertex = a_first_vertex.clone();
    for i in 0..vec_curve.len() {
        let a_next_vertex = next_vertex(&vec_curve, i, (a_surf, a_max_tol, &a_first_vertex))?;
        let an_o_curve = &vec_curve[i];
        let c = an_o_curve.curve.get().ok_or_else(null_curve)?;
        let a_param = (c.first_parameter(), c.last_parameter());
        let curve: Handle<dyn Geom2dCurve> = an_o_curve.curve.clone().cast();
        let an_edge_res: Edge = if an_o_curve.is_reverse {
            let edge = BRepBuilderAPIMakeEdge::from_pcurve_vertices_params(
                &curve,
                &a_surf_h,
                &a_next_vertex,
                &a_vertex,
                a_param.0,
                a_param.1,
            )?
            .edge()?;
            topods::edge(&edge.oriented(Orientation::Reversed))?
        } else {
            BRepBuilderAPIMakeEdge::from_pcurve_vertices_params(
                &curve,
                &a_surf_h,
                &a_vertex,
                &a_next_vertex,
                a_param.0,
                a_param.1,
            )?
            .edge()?
        };
        a_vertex = a_next_vertex;
        a_mk_wire.add_edge(&an_edge_res)?;
    }
    if !a_mk_wire.is_done() {
        return Ok(Wire::default());
    }
    a_mk_wire.wire()
}

/// `BRepAlgo::ConvertFace(theFace, theAngleTolerance)`: the face of its surface bounded
/// by its converted wires (a null face when the maker is not done);
/// `brep_algo::convert_face(&face, 0.1)?`.
pub fn convert_face(the_face: &Face, the_angle_tolerance: f64) -> Result<Face, Failure> {
    let a_surf = BRepTool::surface(the_face)?;
    let mut a_mk_face = BRepBuilderAPIMakeFace::from_surface(&a_surf, precision::confusion())?;
    let mut an_exp = Explorer::new(the_face.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
    while an_exp.more() {
        let a_wire = topods::wire(an_exp.current())?;
        an_exp.next()?;
        let a_new_wire = convert_wire(&a_wire, the_angle_tolerance, the_face)?;
        a_mk_face.add(&a_new_wire)?;
    }
    if !a_mk_face.is_done() {
        return Ok(Face::default());
    }
    a_mk_face.face()
}

/// Standard_NullObject of a null curve or surface (natively a crash).
fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepAlgo: null curve or surface; expected a geometry",
        "BRepAlgo",
    )
}
