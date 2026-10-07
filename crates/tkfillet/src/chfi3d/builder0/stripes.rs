// Rust translation of ChFi3d_HasTransversalIntersection, ChFi3d_HasCommonEndpoint,
// ChFi3d_IndexOfSurfData, ChFi3d_EdgeFromV1, ChFi3d_ConvTol2dToTol3d and
// ChFi3d_EvalTolReached (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The end of a stripe at a vertex, the crossings of two pcurves and the tolerances
//! reached by the edges of a fillet.
use crate::chfi_ds::{StripeHandle, lock};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::TypeTrans;

/// `ChFi3d_HasTransversalIntersection(theIntersector)`: whether a point of the
/// intersection crosses (In or Out on either curve); `has_transversal_intersection(&inter)?`.
pub fn has_transversal_intersection(intersector: &GInter) -> Result<bool, Failure> {
    for index in 1..=intersector.nb_points()? {
        let point = intersector.point(index)?;
        let first = point.transition_of_first().transition_type();
        let second = point.transition_of_second().transition_type();
        if matches!(first, TypeTrans::In | TypeTrans::Out)
            || matches!(second, TypeTrans::In | TypeTrans::Out)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the parameter is an end of the curve (within the tolerance), writing which.
fn is_endpoint(
    parameter: f64,
    curve: &Geom2dAdaptorCurve,
    tolerance: f64,
    start: &mut bool,
) -> Result<bool, Failure> {
    if (parameter - curve.first_parameter()?).abs() <= tolerance {
        *start = true;
        return Ok(true);
    }
    if (parameter - curve.last_parameter()?).abs() <= tolerance {
        *start = false;
        return Ok(true);
    }
    Ok(false)
}

/// `ChFi3d_HasCommonEndpoint(theIntersector, theFirstCurve, theSecondCurve, theTolerance,
/// theFirstCurveStart, theSecondCurveStart)`: whether a point (or a segment shorter than
/// the tolerance) of the intersection is an end of both curves; the start flags are
/// written as they are tested, as natively;
/// `has_common_endpoint(&inter, (&c1, &c2), 1e-7, (&mut s1, &mut s2))?`.
pub fn has_common_endpoint(
    intersector: &GInter,
    (first_curve, second_curve): (&Geom2dAdaptorCurve, &Geom2dAdaptorCurve),
    tolerance: f64,
    (first_start, second_start): (&mut bool, &mut bool),
) -> Result<bool, Failure> {
    for index in 1..=intersector.nb_points()? {
        let point = intersector.point(index)?;
        if is_endpoint(point.param_on_first(), first_curve, tolerance, first_start)?
            && is_endpoint(
                point.param_on_second(),
                second_curve,
                tolerance,
                second_start,
            )?
        {
            return Ok(true);
        }
    }
    for index in 1..=intersector.nb_segments()? {
        let segment = intersector.segment(index)?;
        if !segment.has_first_point() || !segment.has_last_point() {
            continue;
        }
        let (first, last) = (segment.first_point()?, segment.last_point()?);
        if (first.param_on_first() - last.param_on_first()).abs() > tolerance
            || (first.param_on_second() - last.param_on_second()).abs() > tolerance
        {
            continue;
        }
        if is_endpoint(first.param_on_first(), first_curve, tolerance, first_start)?
            && is_endpoint(
                first.param_on_second(),
                second_curve,
                tolerance,
                second_start,
            )?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn wrong_construction() -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        "ChFi3d_IndexOfSurfData() - wrong construction parameters",
    )
}

/// The first edge of the spine of the stripe with its start vertex, and the last edge
/// with its end vertex (as the edges are oriented).
/// An end edge of a spine and its end vertex.
type SpineEnd = (Edge, Vertex);

fn spine_ends(cd: &StripeHandle) -> Result<(SpineEnd, SpineEnd), Failure> {
    let spine = lock(cd).spine().cloned().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "ChFi3d_IndexOfSurfData: null spine")
    })?;
    let spine = lock(&spine);
    let first = spine.spine().edges(1)?.clone();
    let last = spine.spine().edges(spine.spine().nb_edges())?.clone();
    let start = if first.orientation() == Orientation::Reversed {
        topexp::last_vertex(&first, false)
    } else {
        topexp::first_vertex(&first, false)
    };
    let end = if last.orientation() == Orientation::Reversed {
        topexp::first_vertex(&last, false)
    } else {
        topexp::last_vertex(&last, false)
    };
    Ok(((first, start), (last, end)))
}

/// `ChFi3d_IndexOfSurfData(V1, CD, sens)`: `(index, sens)`, the first fillet surface
/// (sens 1) when the stripe starts at the vertex, else the last one (sens -1; index 0
/// without surfaces); Standard_ConstructionError when the stripe does not end there;
/// `index_of_surf_data(&v, &stripe)?`.
pub fn index_of_surf_data(v1: &Vertex, cd: &StripeHandle) -> Result<(i32, i32), Failure> {
    let ((_, start), (_, end)) = spine_ends(cd)?;
    if start.is_same(v1) {
        return Ok((1, 1));
    }
    let Some(surf_data) = lock(cd).set_of_surf_data().cloned() else {
        return Ok((0, -1));
    };
    if !end.is_same(v1) {
        return Err(wrong_construction());
    }
    Ok((lock(&surf_data).len() as i32, -1))
}

/// `ChFi3d_EdgeFromV1(V1, CD, sens)`: `(edge, sens)`, the first edge of the spine with
/// sens 1 when the stripe starts at the vertex, else the last one with sens -1;
/// Standard_ConstructionError when the stripe does not end there;
/// `edge_from_v1(&v, &stripe)?`.
pub fn edge_from_v1(v1: &Vertex, cd: &StripeHandle) -> Result<(Edge, i32), Failure> {
    let ((first, start), (last, end)) = spine_ends(cd)?;
    if start.is_same(v1) {
        return Ok((first, 1));
    }
    if end.is_same(v1) {
        return Ok((last, -1));
    }
    Err(wrong_construction())
}

/// `ChFi3d_ConvTol2dToTol3d(S, tol2d)`: the 3d tolerance of a 2d tolerance by the larger
/// of the U and V resolutions of the surface; `conv_tol2d_to_tol3d(&s, 1e-9)?`.
pub fn conv_tol2d_to_tol3d(s: &Handle<dyn Adaptor3dSurface>, tol2d: f64) -> Result<f64, Failure> {
    let (ures, vres) = (s.u_resolution(1.0e-7)?, s.v_resolution(1.0e-7)?);
    let (uresto3d, vresto3d) = (1.0e-7 * tol2d / ures, 1.0e-7 * tol2d / vres);
    Ok(if uresto3d < vresto3d {
        vresto3d
    } else {
        uresto3d
    })
}

/// `ChFi3d_EvalTolReached(S1, pc1, S2, pc2, C)`: 1.5 times the largest distance between
/// the curve and its two pcurves on 45 parameters (at least the confusion);
/// `eval_tol_reached((&s1, &pc1), (&s2, &pc2), &c)?`.
pub fn eval_tol_reached(
    (s1, pc1): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
    (s2, pc2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Geom2dCurve>),
    c: &Handle<dyn GeomCurve>,
) -> Result<f64, Failure> {
    let (f, l) = (c.first_parameter(), c.last_parameter());
    let nbp = 45;
    let step = 1.0 / f64::from(nbp - 1);
    let mut distmax: f64 = 0.0;
    for i in 0..nbp {
        let t = step * f64::from(i);
        let t = (1.0 - t) * f + t * l;
        let uv = pc1.value(t)?;
        let ps1 = s1.value(uv.x(), uv.y())?;
        let uv = pc2.value(t)?;
        let ps2 = s2.value(uv.x(), uv.y())?;
        let pc = c.value(t)?;
        for d in [
            ps1.square_distance(&pc),
            ps2.square_distance(&pc),
            ps1.square_distance(&ps2),
        ] {
            if d > distmax {
                distmax = d;
            }
        }
    }
    let distmax = 1.5 * distmax.sqrt();
    let confusion = precision::confusion();
    Ok(if distmax < confusion {
        confusion
    } else {
        distmax
    })
}
