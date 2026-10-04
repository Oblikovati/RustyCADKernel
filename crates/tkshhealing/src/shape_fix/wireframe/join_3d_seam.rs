use super::{effective_orientation, is_manifold_orientation, joined_same_vertex};
use crate::shape_construct::{ShapeConstructCurve, ShapeConstructCurveJoin2dRequest};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::top_abs::Orientation;

type SeamCurve = (Handle<Geom2dBSplineCurve>, (f64, f64), bool, bool);
type PcurveData = (Handle<dyn Geom2dCurve>, (f64, f64));
type PcurvePairData = (PcurveData, PcurveData);

pub(super) fn append_seam_pcurves(
    joined: &Edge,
    first: &Edge,
    second: &Edge,
    face: &Face,
    reversed_3d: (bool, bool),
    replace_first: &mut bool,
) -> Result<bool, Failure> {
    let orientations = (
        effective_orientation(first, reversed_3d.0),
        effective_orientation(second, reversed_3d.1),
    );
    let Some(primary) = join_source_pcurves(first, second, face, orientations)? else {
        return Ok(false);
    };
    if is_manifold_pair(first, second)
        && !joined_same_vertex(first, second)
        && (primary.2 || primary.3)
    {
        return Ok(false);
    }
    if primary.2 {
        *replace_first = false;
    }
    let Some(secondary) = join_reversed_pcurves(first, second, face, orientations)? else {
        return Ok(false);
    };
    if !joined_same_vertex(first, second) && (reversed_3d.0 || reversed_3d.1) {
        return Ok(false);
    }
    let pair = (
        primary.0.cast::<dyn Geom2dCurve>(),
        secondary.0.cast::<dyn Geom2dCurve>(),
    );
    install_seam_pcurves(joined, face, &pair, primary.1)
}

fn join_source_pcurves(
    first: &Edge,
    second: &Edge,
    face: &Face,
    orientations: (Orientation, Orientation),
) -> Result<Option<SeamCurve>, Failure> {
    let curves = source_pcurves(first, second, face)?;
    join_curves(curves, orientations, joined_same_vertex(first, second))
}

fn join_reversed_pcurves(
    first: &Edge,
    second: &Edge,
    face: &Face,
    orientations: (Orientation, Orientation),
) -> Result<Option<SeamCurve>, Failure> {
    let first = reversed_edge(first)?;
    let second = reversed_edge(second)?;
    let curves = source_pcurves(&first, &second, face)?;
    join_curves(curves, orientations, joined_same_vertex(&first, &second))
}

fn source_pcurves(first: &Edge, second: &Edge, face: &Face) -> Result<PcurvePairData, Failure> {
    let analysis = crate::shape_analysis::ShapeAnalysisEdge::new();
    let first = analysis.pcurve_on_face(first, face, false)?;
    let second = analysis.pcurve_on_face(second, face, false)?;
    Ok((
        (first.0, (first.1, first.2)),
        (second.0, (second.1, second.2)),
    ))
}

fn join_curves(
    curves: PcurvePairData,
    orientations: (Orientation, Orientation),
    is_error: bool,
) -> Result<Option<SeamCurve>, Failure> {
    if curves.0.0.is_null() || curves.1.0.is_null() {
        return Ok(None);
    }
    let mut request = ShapeConstructCurveJoin2dRequest {
        first: curves.0.0,
        second: curves.1.0,
        first_orientation: orientations.0,
        second_orientation: orientations.1,
        first_range: curves.0.1,
        second_range: curves.1.1,
        is_error,
    };
    let Some(joined) = ShapeConstructCurve.join_curves_2d(&mut request)? else {
        return Ok(None);
    };
    let range = (
        request.first_range.0,
        request.first_range.1 + request.second_range.1 - request.second_range.0,
    );
    Ok(Some((
        joined.curve,
        range,
        joined.reversed_first,
        joined.reversed_second,
    )))
}

fn reversed_edge(edge: &Edge) -> Result<Edge, Failure> {
    let reversed = edge.clone().into_shape().reversed();
    topods::edge(&reversed)
}

fn is_manifold_pair(first: &Edge, second: &Edge) -> bool {
    is_manifold_orientation(first.orientation()) == is_manifold_orientation(second.orientation())
}

fn install_seam_pcurves(
    edge: &Edge,
    face: &Face,
    curves: &(Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>),
    range: (f64, f64),
) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves_on_face(edge, (&curves.0, &curves.1), face, 0.)?;
    builder.range_on_face(edge, face, range.0, range.1)?;
    builder.same_range(edge, false)?;
    Ok(true)
}
