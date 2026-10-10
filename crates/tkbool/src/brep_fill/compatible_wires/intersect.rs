// Rust translation of the static helpers TrimEdge, EdgeIntersectOnWire, Transform and
// BuildConnectedEdges (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The polar method helpers: a point carried from one section plane to the next, the
//! cut of a wire by the ray from its center through that point, and the walk around a
//! wire from an edge.
use super::helpers::{EdgeNewEdges, VertexLists, search_root};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3, Vector3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeWire};
use std::f64::consts::PI;

/// `TrimEdge` of BRepFill_CompatibleWires.cxx: the pieces of the edge at the
/// abscissas `cut_values` (mapped from `[t0, t1]` onto the curve range), from its first
/// vertex when `seq_order`, from its last one otherwise; a piece shorter than
/// Precision::Confusion() in parameter stops the cut and returns the pieces made so far.
pub(super) fn trim_edge(
    edge: &Edge,
    cut_values: &[f64],
    t0: f64,
    t1: f64,
    seq_order: bool,
) -> Result<Vec<Shape>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let orientation = edge.orientation();
    let (vf, vl) = topexp::vertices(edge, false);
    let vbid = Vertex::default();
    let piece = |v0: &Vertex, v1: &Vertex, m0: f64, m1: f64| -> Result<Edge, Failure> {
        let mut cut: Edge =
            BRepLibMakeEdge::from_curve_vertices_params(&curve, v0, v1, m0, m1)?.edge()?;
        cut.set_orientation(orientation);
        Ok(cut)
    };
    let map = |value: f64| (value - t0) * (last - first) / (t1 - t0) + first;
    let ndec: usize = cut_values.len();
    let confusion: f64 = precision::confusion();
    let mut pieces: Vec<Shape> = Vec::new();
    if seq_order {
        // From first to last.
        let (mut m0, mut v0) = (first, vf);
        for (j, &value) in cut_values.iter().enumerate() {
            let m1: f64 = map(value);
            if (m0 - m1).abs() < confusion {
                return Ok(pieces);
            }
            let cut = piece(&v0, &vbid, m0, m1)?;
            m0 = m1;
            v0 = topexp::last_vertex(&cut, false);
            pieces.push(cut.into_shape());
            if j + 1 == ndec {
                if (m0 - last).abs() < confusion {
                    return Ok(pieces);
                }
                pieces.push(piece(&v0, &vl, m0, last)?.into_shape());
            }
        }
        return Ok(pieces);
    }
    // From last to first.
    let (mut m1, mut v1) = (last, vl);
    for j in (0..ndec).rev() {
        let m0: f64 = map(cut_values[j]);
        if (m0 - m1).abs() < confusion {
            return Ok(pieces);
        }
        let cut = piece(&vbid, &v1, m0, m1)?;
        m1 = m0;
        v1 = topexp::first_vertex(&cut, false);
        pieces.push(cut.into_shape());
        if j == 0 {
            if (first - m1).abs() < confusion {
                return Ok(pieces);
            }
            pieces.push(piece(&vf, &v1, first, m1)?.into_shape());
        }
    }
    Ok(pieces)
}

/// The result of `EdgeIntersectOnWire`: whether a new vertex cuts the wire, the
/// vertex found (null when the distance fails) and the cut wire (the wire itself
/// without a cut; none when the distance fails or the cut gives no piece).
pub(super) struct WireIntersection {
    pub(super) new_vertex: bool,
    pub(super) vsol: Vertex,
    pub(super) new_wire: Option<Wire>,
}

/// The support of the BRepExtrema solution nearest to `p2`.
struct NearestSolution {
    isol: i32,
    support: Shape,
    on_vertex: bool,
}

/// The BRepExtrema solution on the wire nearest to `p2` (the first at the least distance).
fn nearest_solution(
    dss: &BRepExtremaDistShapeShape,
    p2: Point3,
) -> Result<NearestSolution, Failure> {
    let mut isol: i32 = 1;
    let mut dss_min: f64 = p2.distance(&dss.point_on_shape2(isol)?);
    for iss in 2..=dss.nb_solution() {
        let distance: f64 = p2.distance(&dss.point_on_shape2(iss)?);
        if dss_min > distance {
            dss_min = distance;
            isol = iss;
        }
    }
    Ok(NearestSolution {
        isol,
        support: dss.support_on_shape2(isol)?,
        on_vertex: dss.support_type_shape2(isol)? == BRepExtremaSupportType::IsVertex,
    })
}

/// `EdgeIntersectOnWire(P1, P2, percent, Map, W, Vsol, newW, theEdgeNewEdges)`: the
/// point of the wire nearest to the segment from `p1` along `p1 p2` (as long as the
/// diagonal of the wire box): a vertex of the wire, an edge end within `percent` of
/// its range (unless it is already a root of `map`), else a new vertex cutting the
/// edge, recorded in `new_edges`.
pub(super) fn edge_intersect_on_wire(
    p1: Point3,
    p2: Point3,
    percent: f64,
    map: &VertexLists,
    wire: &Wire,
    new_edges: &mut EdgeNewEdges,
) -> Result<WireIntersection, Failure> {
    let droite = Line3::new(p1, Direction3::from_vector(Vector3::between(p1, p2))?);
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(wire, &mut bounds, true)?;
    let [x1, y1, z1, x2, y2, z2] = bounds.get()?;
    let diag: f64 = Point3::new(x1, y1, z1).distance(&Point3::new(x2, y2, z2));
    let me = BRepLibMakeEdge::from_lin_params(droite, 0., diag)?;
    // The segment of the native `ECur` (built, then never used).
    let _ = BRepLibMakeEdge::from_lin_params(droite, 0., p1.distance(&p2))?.edge()?;
    let dss =
        BRepExtremaDistShapeShape::with_shapes(&me.edge()?.into_shape(), wire, Default::default())?;
    let mut result = WireIntersection {
        new_vertex: false,
        vsol: Vertex::default(),
        new_wire: None,
    };
    if !dss.is_done() {
        return Ok(result);
    }
    let solution = nearest_solution(&dss, p2)?;
    result.new_vertex = !solution.on_vertex;
    if solution.on_vertex {
        result.vsol = topods::vertex(&solution.support)?;
    } else {
        settle_edge_end(&dss, &solution, percent, map, &mut result)?;
    }
    if !result.new_vertex {
        result.new_wire = Some(wire.clone());
        return Ok(result);
    }
    cut_wire_at(&dss, &solution, wire, new_edges, &mut result)?;
    Ok(result)
}

/// An edge solution near an end of the edge (within `percent` of its range) is that
/// end vertex, unless the vertex is already reported in `map`.
fn settle_edge_end(
    dss: &BRepExtremaDistShapeShape,
    solution: &NearestSolution,
    percent: f64,
    map: &VertexLists,
    result: &mut WireIntersection,
) -> Result<(), Failure> {
    let edge: Edge = topods::edge(&solution.support)?;
    let (first, last) = BRepTool::range(&edge)?;
    let tol: f64 = precision::p_confusion().max(percent * (last - first).abs());
    let param: f64 = dss.par_on_edge_s2(solution.isol)?;
    if (first - param).abs() < tol {
        result.new_vertex = false;
        result.vsol = topexp::first_vertex(&edge, false);
    } else if (last - param).abs() < tol {
        result.new_vertex = false;
        result.vsol = topexp::last_vertex(&edge, false);
    }
    // Check.
    if !result.new_vertex && search_root(&result.vsol, map).is_some() {
        result.new_vertex = true;
    }
    Ok(())
}

/// The wire rebuilt with the supporting edge cut at the solution parameter; the new
/// vertex is the end of the first piece that is not an end of the edge.
fn cut_wire_at(
    dss: &BRepExtremaDistShapeShape,
    solution: &NearestSolution,
    wire: &Wire,
    new_edges: &mut EdgeNewEdges,
    result: &mut WireIntersection,
) -> Result<(), Failure> {
    let edge: Edge = topods::edge(&solution.support)?;
    new_edges.bind(edge.as_ref().clone(), Vec::new());
    let param: f64 = dss.par_on_edge_s2(solution.isol)?;
    let (first, last) = BRepTool::range(&edge)?;
    let mut maker = BRepLibMakeWire::new();
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        if !edge.is_same(explorer.current()) {
            maker.add_edge(explorer.current())?;
            explorer.next()?;
            continue;
        }
        let so: bool = explorer
            .current_vertex()
            .is_same(&topexp::first_vertex(&edge, false));
        let pieces: Vec<Shape> = trim_edge(&edge, &[param], first, last, so)?;
        if pieces.is_empty() {
            result.new_vertex = false;
            return Ok(());
        }
        *new_edges.change_find(&edge)? = pieces.clone();
        let (vv1, vv2) = topexp::vertices(&topods::edge(&pieces[0])?, false);
        let (ef, el) = (
            topexp::first_vertex(&edge, false),
            topexp::last_vertex(&edge, false),
        );
        if ef.is_same(&vv1) || el.is_same(&vv1) {
            result.vsol = vv2.clone();
        }
        if ef.is_same(&vv2) || el.is_same(&vv2) {
            result.vsol = vv1;
        }
        for piece in &pieces {
            maker.add_edge(&topods::edge(piece)?)?;
        }
        explorer.next()?;
    }
    result.new_wire = Some(maker.wire()?);
    Ok(())
}

/// `Transform(WithRotation, P, Pos1, Ax1, Pos2, Ax2, Pnew)`: `p` translated from the
/// plane `(pos1, ax1)` to the plane `(pos2, ax2)`, then rotated about the normal of
/// both axes by their angle (within +-PI/2 without rotation), with a sign set by the
/// position of the planes.
pub(super) fn transform(
    with_rotation: bool,
    p: Point3,
    plane1: (Point3, Vector3),
    plane2: (Point3, Vector3),
) -> Result<Point3, Failure> {
    let (pos1, ax1) = plane1;
    let (pos2, ax2) = plane2;
    let pnew: Point3 = p.translated_between(pos1, pos2);
    let (mut axe1, mut axe2) = (ax1, ax2);
    if axe1.is_parallel(axe2, 1.0e-4)? {
        return Ok(pnew);
    }
    let vtrans = Vector3::between(pos1, pos2);
    if vtrans.dot(axe1) < -1.0e-7 {
        axe1 *= -1.;
    }
    if vtrans.dot(axe2) < 1.0e-7 {
        axe2 *= -1.;
    }
    let norm2: Vector3 = axe1.crossed(axe2);
    let mut vsign = Vector3::default();
    vsign.set_linear_form(vtrans.dot(axe1), axe2, -vtrans.dot(axe2), axe1);
    let (alpha, beta) = (vsign.dot(axe1), vsign.dot(axe2));
    let pasnul: bool = alpha.abs() > 1.0e-4 && beta.abs() > 1.0e-4;
    let sign: f64 = if alpha * beta > 0.0 && pasnul {
        -1.
    } else {
        1.
    };
    let norm = Axis1::new(pos2, Direction3::from_vector(norm2)?);
    let mut ang: f64 = axe1.angle_with_ref(axe2, norm2)?;
    if !with_rotation {
        if ang > PI / 2. {
            ang -= PI;
        }
        if ang < -PI / 2. {
            ang += PI;
        }
    }
    ang *= sign;
    pnew.rotated(norm, ang)
}

/// `BuildConnectedEdges`: the edges met walking around the wire from `start_vertex`
/// away from `start_edge`, until the other vertex of `start_edge` is reached.
pub(super) fn build_connected_edges(
    wire: &Wire,
    start_edge: &Edge,
    start_vertex: &Vertex,
) -> Result<Vec<Shape>, Failure> {
    let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map_ve)?;
    let mut cur_edge: Edge = start_edge.clone();
    let mut cur_vertex: Vertex = start_vertex.clone();
    let (v1, v2) = topexp::vertices(start_edge, false);
    let origin: Vertex = if v1.is_same(start_vertex) { v2 } else { v1 };
    let mut connected: Vec<Shape> = Vec::new();
    loop {
        let candidates = map_ve.find_from_key(&cur_vertex)?.clone();
        for candidate in candidates.iter() {
            let an_edge: Edge = topods::edge(candidate)?;
            if !an_edge.is_same(&cur_edge) {
                connected.push(an_edge.as_ref().clone());
                let (e1, e2) = topexp::vertices(&an_edge, false);
                cur_vertex = if e1.is_same(&cur_vertex) { e2 } else { e1 };
                cur_edge = an_edge;
                break;
            }
        }
        if cur_vertex.is_same(&origin) {
            return Ok(connected);
        }
    }
}
