// Rust translation of ShapeConstruct::JoinPCurves (ShapeConstruct.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeConstruct::JoinPCurves`: puts on an edge the join of the p-curves of a chain.
use super::ShapeConstruct;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_construct::{ShapeConstructCurve, ShapeConstructCurveJoin2dRequest};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::Orientation;

/// The joined p-curves accumulated over the chain.
struct JoinedPCurves {
    first: Handle<dyn Geom2dCurve>,
    second: Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    seam: bool,
}

impl ShapeConstruct {
    /// `JoinPCurves(edges, face, edge)`: joins the p-curves (both seam p-curves on a seam)
    /// of the chain `edges` on `face` and stores the result on `edge` with its range,
    /// clearing SameRange and SameParameter. A plane needs no p-curve: true at once.
    ///
    /// Native quirk (kept): the result is `i <= Length`, so it is true when the join
    /// stopped early (mixed seams, a missing p-curve, a failed join) and false when every
    /// edge was joined; the p-curves joined so far are stored either way. A raised
    /// exception returns false without the store. Example:
    /// `let stopped_early = ShapeConstruct::join_pcurves(&edges, &face, &mut edge);`.
    pub fn join_pcurves(edges: &Sequence<Shape>, face: &Face, edge: &mut Edge) -> bool {
        join_pcurves_or_raise(edges, face, edge).unwrap_or(false)
    }
}

fn join_pcurves_or_raise(
    edges: &Sequence<Shape>,
    face: &Face,
    edge: &mut Edge,
) -> Result<bool, Failure> {
    if face_is_plane(face)? {
        return Ok(true);
    }
    let mut joined = JoinedPCurves {
        first: Handle::default(),
        second: Handle::default(),
        range: (0., 0.),
        seam: false,
    };
    let mut index = 1;
    while index <= edges.len() {
        let current = topods::edge(edges.value(index)?)?;
        if !join_next(&mut joined, &current, face, index == 1)? {
            break;
        }
        index += 1;
    }
    store_pcurves(&joined, face, edge)?;
    Ok(index <= edges.len())
}

fn face_is_plane(face: &Face) -> Result<bool, Failure> {
    let mut surface = BRepTool::surface(face)?;
    while let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        surface = trimmed.basis_surface();
    }
    Ok(!surface.cast::<GeomPlane>().is_null())
}

// One iteration of the native loop; false is the native `break`.
fn join_next(
    joined: &mut JoinedPCurves,
    edge: &Edge,
    face: &Face,
    first_edge: bool,
) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let is_seam = analysis.is_seam_on_face(edge, face)?;
    if first_edge {
        joined.seam = is_seam;
    } else if joined.seam != is_seam {
        return Ok(false);
    }
    let (c2d, first, last) = analysis.pcurve_on_face(edge, face, false)?;
    if c2d.is_null() {
        return Ok(false);
    }
    let mut reversed = edge.clone();
    reversed.reverse();
    let (c2d2, first2, last2) = if joined.seam {
        analysis.pcurve_on_face(&reversed, face, false)?
    } else {
        (Handle::default(), 0., 0.)
    };
    if first_edge {
        *joined = JoinedPCurves {
            first: c2d,
            second: c2d2,
            range: (first, last),
            seam: joined.seam,
        };
        return Ok(true);
    }
    append_pcurves(
        joined,
        edge.orientation(),
        (c2d, first, last),
        (c2d2, first2, last2),
    )
}

// Native resets resOrient to FORWARD at the top of every iteration, so the accumulated
// curve always joins as FORWARD after the first edge (kept as is).
fn append_pcurves(
    joined: &mut JoinedPCurves,
    orientation: Orientation,
    next: (Handle<dyn Geom2dCurve>, f64, f64),
    next_seam: (Handle<dyn Geom2dCurve>, f64, f64),
) -> Result<bool, Failure> {
    let mut request = join_request(&joined.first, joined.range, &next, orientation);
    let Some(curve) = ShapeConstructCurve.join_curves_2d(&mut request)?.curve else {
        return Ok(false);
    };
    let (first, last) = request.second_range;
    if joined.seam {
        // The native seam join starts from the range the first join left (newf, newl).
        let mut seam = join_request(&joined.second, request.first_range, &next_seam, orientation);
        let Some(seam_curve) = ShapeConstructCurve.join_curves_2d(&mut seam)?.curve else {
            return Ok(false);
        };
        joined.second = seam_curve;
    }
    let (fp2d, lp2d) = (curve.first_parameter(), curve.last_parameter());
    joined.first = curve;
    let (mut newf, mut newl) = request.first_range;
    newl += last - first;
    if fp2d > newf {
        newf = fp2d;
    }
    if lp2d < newl {
        newl = lp2d;
    }
    joined.range = (newf, newl);
    Ok(true)
}

fn join_request(
    accumulated: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    next: &(Handle<dyn Geom2dCurve>, f64, f64),
    orientation: Orientation,
) -> ShapeConstructCurveJoin2dRequest {
    ShapeConstructCurveJoin2dRequest {
        first: accumulated.clone(),
        second: next.0.clone(),
        first_orientation: Orientation::Forward,
        second_orientation: orientation,
        first_range: range,
        second_range: (next.1, next.2),
        is_error: false,
    }
}

fn store_pcurves(joined: &JoinedPCurves, face: &Face, edge: &Edge) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if joined.seam {
        builder.update_edge_pcurves_on_face(edge, (&joined.first, &joined.second), face, 0.)?;
    } else {
        builder.update_edge_pcurve_on_face(edge, &joined.first, face, 0.)?;
    }
    builder.range_on_face(edge, face, joined.range.0, joined.range.1)?;
    builder.same_range(edge, false)?;
    builder.same_parameter(edge, false)
}
