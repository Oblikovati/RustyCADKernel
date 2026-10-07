// Rust translation of the static JoinEdges of ShapeFix_Wireframe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `JoinEdges(E1, E2, E3, faces)`: two consecutive edges joined into one, their 3D
//! curves and their p-curves on the faces concatenated by ShapeConstruct::JoinCurves.
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_construct::{ShapeConstructCurve, ShapeConstructCurveJoin2dRequest};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::top_abs::Orientation;

/// The result of `JoinEdges`: the new edge (`E3`, null when the edges cannot be joined)
/// and `ReplaceFirst` (whether the new edge replaces the first edge).
pub(super) struct JoinedEdges {
    pub edge: Edge,
    pub replace_first: bool,
}

/// The joined edge under construction and the orientations used for the p-curves.
struct JoinState {
    edge: Edge,
    replace_first: bool,
    is_same: bool,
    manifold: bool,
    orientations: (Orientation, Orientation),
}

fn is_manifold(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

/// The orientation of an edge after the 3D join reversed it (or not).
fn joined_orientation(edge: &Edge, reversed: bool) -> Orientation {
    let forward = edge.orientation() == Orientation::Forward;
    let reversed_edge = edge.orientation() == Orientation::Reversed;
    if (!reversed && forward) || (reversed && reversed_edge) {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

/// `JoinEdges(E1, E2, E3, faces)`: an exception aborts the join with the `ReplaceFirst`
/// computed so far and a null edge, as the native try/catch.
pub(super) fn join_edges(e1: &Edge, e2: &Edge, faces: &[Shape]) -> JoinedEdges {
    let mut replace_first = true;
    let edge = match join_body(e1, e2, faces, &mut replace_first) {
        Ok(Some(edge)) => edge,
        Ok(None) | Err(_) => Edge::default(),
    };
    JoinedEdges {
        edge,
        replace_first,
    }
}

fn join_body(
    e1: &Edge,
    e2: &Edge,
    faces: &[Shape],
    replace_first: &mut bool,
) -> Result<Option<Edge>, Failure> {
    let sae = ShapeAnalysisEdge::new();
    let (c3d1, cf1, cl1) = sae.curve_3d(e1, false)?;
    let (c3d2, cf2, cl2) = sae.curve_3d(e2, false)?;
    if c3d1.is_null() || c3d2.is_null() {
        return Ok(None);
    }
    let mut state = new_joined_edge(e1, e2)?;
    let joined = join_3d(e1, e2, (c3d1, (cf1, cl1)), (c3d2, (cf2, cl2)), &mut state)?;
    *replace_first = state.replace_first;
    if !joined {
        return Ok(None);
    }
    for face in faces {
        let ok = join_on_face(e1, e2, &topods::face(face)?, &mut state)?;
        *replace_first = state.replace_first;
        if !ok {
            return Ok(None);
        }
    }
    BRepBuilder::default().same_range(&state.edge, false)?;
    Ok(Some(state.edge))
}

/// The new forward edge from the first vertex of `E1` to the last vertex of `E2`;
/// `isSame` when one of the edges is closed on its vertex.
fn new_joined_edge(e1: &Edge, e2: &Edge) -> Result<JoinState, Failure> {
    let sae = ShapeAnalysisEdge::new();
    let (v11, v12) = (sae.first_vertex(e1), sae.last_vertex(e1));
    let (v21, v22) = (sae.first_vertex(e2), sae.last_vertex(e2));
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge)?;
    edge.set_orientation(Orientation::Forward);
    let mut shape = edge.into_shape();
    Builder.add(&mut shape, &v11.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &v22.oriented(Orientation::Reversed))?;
    Ok(JoinState {
        edge: topods::edge(&shape)?,
        replace_first: true,
        is_same: v11.is_same(&v12) || v22.is_same(&v21),
        manifold: false,
        orientations: (e1.orientation(), e2.orientation()),
    })
}

/// The 3D part: the joined curve on the composed range; false when JoinCurves fails,
/// reverses an edge without a shared vertex, or mixes manifold and non-manifold edges.
fn join_3d(
    e1: &Edge,
    e2: &Edge,
    (c3d1, mut r1): (Handle<dyn GeomCurve>, (f64, f64)),
    (c3d2, mut r2): (Handle<dyn GeomCurve>, (f64, f64)),
    state: &mut JoinState,
) -> Result<bool, Failure> {
    let (o1, o2) = (e1.orientation(), e2.orientation());
    let joined = ShapeConstructCurve.join_curves_3d(&c3d1, &c3d2, o1, o2, &mut r1, &mut r2)?;
    let Some(joined) = joined else {
        return Ok(false);
    };
    let (rev1, rev2) = (joined.reversed_first, joined.reversed_second);
    if !state.is_same && (rev1 || rev2) {
        return Ok(false);
    }
    state.replace_first = !rev1;
    state.manifold = is_manifold(o1);
    if state.manifold != is_manifold(o2) {
        return Ok(false);
    }
    if state.manifold {
        state.orientations = (joined_orientation(e1, rev1), joined_orientation(e2, rev2));
    }
    let tolerance = BRepTool::edge_tolerance(e1)?.max(BRepTool::edge_tolerance(e2)?);
    install_curve(
        &state.edge,
        &joined.curve,
        tolerance,
        (r1.0, r1.1 + r2.1 - r2.0),
    )?;
    Ok(true)
}

/// The joined 3D curve and its range (the composed range bounded by the curve's).
fn install_curve(
    edge: &Edge,
    joined: &Handle<GeomBSplineCurve>,
    tolerance: f64,
    (newf, newl): (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let curve: Handle<dyn GeomCurve> = joined.clone().cast();
    builder.update_edge_curve(edge, &curve, tolerance)?;
    let c = joined.get().ok_or_else(null_curve)?;
    builder.range(
        edge,
        newf.max(c.first_parameter()),
        newl.min(c.last_parameter()),
        false,
    )
}

/// The joined p-curve (or seam pair) on one face; false when a p-curve is missing, the
/// join fails, or only one edge is a seam.
fn join_on_face(e1: &Edge, e2: &Edge, face: &Face, state: &mut JoinState) -> Result<bool, Failure> {
    let Some((curve, (rev12, rev22), (first1, last1), (first2, last2))) =
        join_pcurves(e1, e2, face, state)?
    else {
        return Ok(false);
    };
    if state.manifold && !state.is_same && (rev12 || rev22) {
        return Ok(false);
    }
    if rev12 {
        state.replace_first = false;
    }
    let c = curve.get().ok_or_else(null_curve)?;
    let newf1 = first1.max(c.first_parameter());
    let newl1 = (last1 + (last2 - first2)).min(c.last_parameter());
    if !install_pcurves(e1, e2, face, curve.cast(), state)? {
        return Ok(false);
    }
    BRepBuilder::default().range_on_face(&state.edge, face, newf1, newl1)?;
    if !state.manifold {
        let (o1, o2) = state.orientations;
        state
            .edge
            .set_orientation(if state.replace_first { o1 } else { o2 });
    }
    Ok(true)
}

/// The joined p-curve on the face, or both joined p-curves when both edges are seams
/// (the second from the reversed edges); false when only one edge is a seam.
fn install_pcurves(
    e1: &Edge,
    e2: &Edge,
    face: &Face,
    curve: Handle<dyn Geom2dCurve>,
    state: &JoinState,
) -> Result<bool, Failure> {
    let sae = ShapeAnalysisEdge::new();
    let (seam1, seam2) = (
        sae.is_seam_on_face(e1, face)?,
        sae.is_seam_on_face(e2, face)?,
    );
    let builder = BRepBuilder::default();
    if seam1 != seam2 {
        return Ok(false);
    }
    if !seam1 {
        builder.update_edge_pcurve_on_face(&state.edge, &curve, face, 0.)?;
        return Ok(true);
    }
    let (mut r1, mut r2) = (e1.clone(), e2.clone());
    r1.reverse();
    r2.reverse();
    // The native test of the 3D reversal flags that follows this join is always false
    // here (JoinEdges returned earlier otherwise).
    let Some((second, _, _, _)) = join_pcurves(&r1, &r2, face, state)? else {
        return Ok(false);
    };
    let second: Handle<dyn Geom2dCurve> = second.cast();
    builder.update_edge_pcurves_on_face(&state.edge, (&curve, &second), face, 0.)?;
    Ok(true)
}

type JoinedPCurve = (
    Handle<Geom2dBSplineCurve>,
    (bool, bool),
    (f64, f64),
    (f64, f64),
);

/// ShapeConstruct::JoinCurves of the p-curves of the edges on the face (not oriented),
/// with the updated ranges.
fn join_pcurves(
    e1: &Edge,
    e2: &Edge,
    face: &Face,
    state: &JoinState,
) -> Result<Option<JoinedPCurve>, Failure> {
    let sae = ShapeAnalysisEdge::new();
    let (c2d1, first1, last1) = sae.pcurve_on_face(e1, face, false)?;
    if c2d1.is_null() {
        return Ok(None);
    }
    let (c2d2, first2, last2) = sae.pcurve_on_face(e2, face, false)?;
    if c2d2.is_null() {
        return Ok(None);
    }
    let mut request = ShapeConstructCurveJoin2dRequest {
        first: c2d1,
        second: c2d2,
        first_orientation: state.orientations.0,
        second_orientation: state.orientations.1,
        first_range: (first1, last1),
        second_range: (first2, last2),
        is_error: state.is_same,
    };
    let Some(joined) = ShapeConstructCurve.join_curves_2d(&mut request)? else {
        return Ok(None);
    };
    Ok(Some((
        joined.curve,
        (joined.reversed_first, joined.reversed_second),
        request.first_range,
        request.second_range,
    )))
}

fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeFix_Wireframe::JoinEdges: null joined curve; expected a B-spline",
        "",
    )
}
