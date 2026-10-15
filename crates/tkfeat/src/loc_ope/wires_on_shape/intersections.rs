// Rust translation of the static FindInternalIntersections of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The split of a splitting edge where it crosses an edge of its face away from its
//! vertices, or the detection that it overlaps one.
use crate::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::extrema::ExtCC;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// `ext`: 4 * 4, the squared factor of the vertex tolerances around which no split is
/// made, to avoid creating microedges (as the Boolean operations do).
const EXT: f64 = 16.;

/// What the analysed edge brings to the search: its vertices, their points and
/// tolerances, its curve and range, and the 2D box of its p-curve.
struct SplitEdge {
    vertices: [Vertex; 2],
    pnt: [Point3; 2],
    tol_v_ext: [f64; 2],
    tol_v2d: [f64; 2],
    curve: Handle<dyn GeomCurve>,
    par: [f64; 2],
    bounds: BoundingBox2,
}

/// The 2D box of the p-curve of `edge` on `face`, enlarged by the edge tolerance.
fn pcurve_box(edge: &Edge, face: &Face) -> Result<BoundingBox2, Failure> {
    let pcurve = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let mut bounds = BoundingBox2::default();
    Add2dCurve::add_adaptor(
        pcurve.geom2d_adaptor(),
        BRepTool::edge_tolerance(edge)?,
        &mut bounds,
    )?;
    Ok(bounds)
}

impl SplitEdge {
    fn new(edge: &Edge, face: &Face) -> Result<Self, Failure> {
        let (v0, v1) = topexp::vertices(edge, false);
        let tol_v = [
            BRepTool::vertex_tolerance(&v0)?,
            BRepTool::vertex_tolerance(&v1)?,
        ];
        let pnt = [BRepTool::pnt(&v0)?, BRepTool::pnt(&v1)?];
        let (curve, f, l) = BRepTool::curve(edge)?;
        let ga_curve = GeomAdaptorCurve::with_range(&curve, f, l)?;
        let tol_v2d = [
            ga_curve.resolution(tol_v[0])?.max(precision::p_confusion()),
            ga_curve.resolution(tol_v[1])?.max(precision::p_confusion()),
        ];
        Ok(Self {
            vertices: [v0, v1],
            pnt,
            tol_v_ext: [EXT * tol_v[0] * tol_v[0], EXT * tol_v[1] * tol_v[1]],
            tol_v2d,
            curve,
            par: [f, l],
            bounds: pcurve_box(edge, face)?,
        })
    }

    fn curve(&self) -> Result<&dyn GeomCurve, Failure> {
        self.curve.get().ok_or_else(|| {
            null_object(
                "LocOpe_WiresOnShape::FindInternalIntersections",
                "edge without 3D curve",
            )
        })
    }

    /// Whether both points of a crossing are farther than the enlarged tolerance of
    /// both vertices.
    fn away_from_vertices(&self, a: Point3, b: Point3) -> bool {
        a.square_distance(&self.pnt[0]) > self.tol_v_ext[0]
            && a.square_distance(&self.pnt[1]) > self.tol_v_ext[1]
            && b.square_distance(&self.pnt[0]) > self.tol_v_ext[0]
            && b.square_distance(&self.pnt[1]) > self.tol_v_ext[1]
    }
}

/// What one edge of the face does to the analysed edge.
enum Crossing {
    /// Keep searching (no extremum, or crossings appended).
    Continue,
    /// The edges overlap within the tolerance of the face edge.
    Overlapped,
    /// An end of the edges lies near a vertex: the native search stops without split.
    Stop,
}

/// The parameters on the analysed edge where the face edge `an_edge` crosses it away
/// from its vertices, appended to `split_pars`; the largest squared distance in
/// `dist_max`.
fn cross_edge(
    se: &SplitEdge,
    an_edge: &Edge,
    face: &Face,
    split_pars: &mut Vec<f64>,
    dist_max: &mut f64,
) -> Result<Crossing, Failure> {
    let tol_ext = precision::p_confusion();
    if se.bounds.is_out_box(&pcurve_box(an_edge, face)?) {
        return Ok(Crossing::Continue);
    }
    let (a_curve, af, al) = BRepTool::curve(an_edge)?;
    let the_ga = GeomAdaptorCurve::with_range(&se.curve, se.par[0], se.par[1])?;
    let a_ga = GeomAdaptorCurve::with_range(&a_curve, af, al)?;
    let extrema = ExtCC::with_curves(&the_ga, &a_ga, tol_ext, tol_ext)?;
    if !extrema.is_done() || extrema.nb_ext()? == 0 {
        return Ok(Crossing::Continue);
    }
    let max_tol2 = BRepTool::edge_tolerance(an_edge)?.powi(2);
    if extrema.is_parallel()? && extrema.square_distance(1)? <= max_tol2 {
        return Ok(Crossing::Overlapped);
    }
    // Check extremity distances
    let (dists, _) = extrema.trimmed_square_distances();
    for (i, dist) in dists.iter().enumerate() {
        let j = if i < 2 { 0 } else { 1 };
        if *dist < se.tol_v_ext[j] / EXT {
            return Ok(Crossing::Stop);
        }
    }
    let a_curve = a_curve.get().ok_or_else(|| {
        null_object(
            "LocOpe_WiresOnShape::FindInternalIntersections",
            "face edge without 3D curve",
        )
    })?;
    for i in 1..=extrema.nb_ext()? {
        let dist = extrema.square_distance(i)?;
        if dist > max_tol2 {
            continue;
        }
        let (p1, p2) = extrema.points(i)?;
        let (the_int_par, an_int_par) = (p1.parameter(), p2.parameter());
        // try to find intersection on an extremity of "theEdge"
        if (0..2).any(|j| (the_int_par - se.par[j]).abs() <= se.tol_v2d[j]) {
            continue;
        }
        // intersection is inside "theEdge" => split
        let point = a_curve.value(an_int_par)?;
        let point_int = se.curve()?.value(the_int_par)?;
        if se.away_from_vertices(point_int, point) {
            split_pars.push(the_int_par);
            if dist > *dist_max {
                *dist_max = dist;
            }
        }
    }
    Ok(Crossing::Continue)
}

/// The split parameters sorted (the native exchange sort) and stripped of those whose
/// points coincide with the previous one.
fn sorted_distinct(se: &SplitEdge, mut pars: Vec<f64>) -> Result<Vec<f64>, Failure> {
    let n = pars.len();
    for i in 0..n.saturating_sub(1) {
        for j in i + 1..n {
            if pars[i] > pars[j] {
                pars.swap(i, j);
            }
        }
    }
    let mut i = 0;
    while i + 1 < pars.len() {
        let pnt1 = se.curve()?.value(pars[i])?;
        let pnt2 = se.curve()?.value(pars[i + 1])?;
        if pnt1.square_distance(&pnt2) <= precision::confusion() * precision::confusion() {
            pars.remove(i + 1);
        } else {
            i += 1;
        }
    }
    Ok(pars)
}

/// The piece of `the_edge` between `first` and `last` (FORWARD and REVERSED), with its
/// orientation, its tolerance raised to the same-parameter deviation on the face.
fn piece(
    the_edge: &Edge,
    face: &Face,
    first: (&Vertex, f64),
    last: (&Vertex, f64),
) -> Result<Edge, Failure> {
    let bb = BRepBuilder::default();
    let mut local = the_edge.empty_copied()?;
    let orient = local.orientation();
    local.set_orientation(Orientation::Forward);
    let mut new_edge = topods::edge(&local)?;
    bb.range(&new_edge, first.1, last.1, false)?;
    bb.add(&mut new_edge, &first.0.oriented(Orientation::Forward))?;
    bb.add(&mut new_edge, &last.0.oriented(Orientation::Reversed))?;
    new_edge.set_orientation(orient);
    let mut sae = ShapeAnalysisEdge::default();
    let (done, maxdev) = sae.check_same_parameter_on_face(&new_edge, face, 23)?;
    if done {
        bb.update_edge_tolerance(&new_edge, maxdev)?;
    }
    Ok(new_edge)
}

/// The pieces of the edge at the split parameters, new vertices of tolerance
/// `sqrt(dist_max)` at the cuts, in the order of the edge orientation.
fn split_pieces(
    the_edge: &Edge,
    face: &Face,
    se: &SplitEdge,
    pars: &[f64],
    dist_max: f64,
) -> Result<List<Shape>, Failure> {
    let mut new_edges = List::default();
    let mut first_vertex = se.vertices[0].clone();
    let mut first_par = se.par[0];
    for i in 0..=pars.len() {
        let (last_vertex, last_par) = if i < pars.len() {
            let last_point = se.curve()?.value(pars[i])?;
            let vertex = BRepLibMakeVertex::new(last_point)?.vertex()?;
            BRepBuilder::default().update_vertex_tolerance(&vertex, dist_max.sqrt())?;
            (vertex, pars[i])
        } else {
            (se.vertices[1].clone(), se.par[1])
        };
        let new_edge = piece(
            the_edge,
            face,
            (&first_vertex, first_par),
            (&last_vertex, last_par),
        )?;
        if new_edge.orientation() == Orientation::Forward {
            new_edges.append(new_edge.into_shape());
        } else {
            new_edges.prepend(new_edge.into_shape());
        }
        first_vertex = last_vertex;
        first_par = last_par;
    }
    Ok(new_edges)
}

/// `FindInternalIntersections(theEdge, theFace, Splits, isOverlapped)`: the pieces of
/// `the_edge` cut where the edges of `the_face` cross it inside, added to `splits`;
/// `is_overlapped` when it overlaps one of them.
pub(super) fn find_internal_intersections(
    the_edge: &Edge,
    the_face: &Face,
    splits: &mut IndexedDataMapOfShapeListOfShape,
    is_overlapped: &mut bool,
) -> Result<(), Failure> {
    let se = SplitEdge::new(the_edge, the_face)?;
    let mut split_pars = Vec::new();
    let mut dist_max = precision::confusion() * precision::confusion();
    for e in Explorer::new(the_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let an_edge = topods::edge(&e)?;
        match cross_edge(&se, &an_edge, the_face, &mut split_pars, &mut dist_max)? {
            Crossing::Continue => {}
            Crossing::Overlapped => {
                *is_overlapped = true;
                return Ok(());
            }
            Crossing::Stop => return Ok(()),
        }
    }
    if split_pars.is_empty() {
        return Ok(());
    }
    let pars = sorted_distinct(&se, split_pars)?;
    let new_edges = split_pieces(the_edge, the_face, &se, &pars, dist_max)?;
    if !new_edges.is_empty() {
        splits.add(the_edge.as_ref().clone(), new_edges);
    }
    Ok(())
}
