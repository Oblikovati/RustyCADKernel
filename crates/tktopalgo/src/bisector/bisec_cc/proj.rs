// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (the static ProjOnCurve and TestExtension) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projection of a point on a curve (its ends first), and the test of an
//! extension at a common end of two curves.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `ProjOnCurve(P, C, theParam)`: the end of the curve that `p` is on or normal to,
/// else the lowest-distance projection; `None` when there is none.
pub(crate) fn proj_on_curve(
    p: Point2,
    c: &Handle<dyn Geom2dCurve>,
) -> Result<Option<f64>, Failure> {
    let (first, last) = (c.first_parameter(), c.last_parameter());
    let (pf, mut tf) = c.d1(first)?;
    let (pl, mut tl) = c.d1(last)?;
    let conf: f64 = precision::confusion();
    if p.is_equal(pf, conf) {
        return Ok(Some(first));
    }
    if p.is_equal(pl, conf) {
        return Ok(Some(last));
    }
    let ppf = Vector2::new(pf.x() - p.x(), pf.y() - p.y());
    tf.normalize()?;
    if ppf.dot(tf).abs() < conf {
        return Ok(Some(first));
    }
    let ppl = Vector2::new(pl.x() - p.x(), pl.y() - p.y());
    tl.normalize()?;
    if ppl.dot(tl).abs() < conf {
        return Ok(Some(last));
    }
    let projection = ProjectPointOnCurve::with_range(p, c, first, last)?;
    if projection.nb_points() > 0 {
        return Ok(Some(projection.lower_distance_parameter()?));
    }
    Ok(None)
}

/// `TestExtension(C1, C2, Start_End)`: true when the start (1) or end (2) of `c1`
/// is an end of `c2` with the same tangent direction. At the last end of `c2` the
/// tangent of `c1` is compared without being normalized, as natively.
pub(crate) fn test_extension(
    c1: &Handle<dyn Geom2dCurve>,
    c2: &Handle<dyn Geom2dCurve>,
    start_end: i32,
) -> Result<bool, Failure> {
    let u1: f64 = if start_end == 1 {
        c1.first_parameter()
    } else {
        c1.last_parameter()
    };
    let (p1, mut t1) = c1.d1(u1)?;
    let conf: f64 = precision::confusion();
    let (p2, mut t2) = c2.d1(c2.first_parameter())?;
    if p1.is_equal(p2, conf) {
        t1.normalize()?;
        t2.normalize()?;
        return Ok(t1.dot(t2) > 1.0 - conf);
    }
    let (p2, mut t2) = c2.d1(c2.last_parameter())?;
    if p1.is_equal(p2, conf) {
        t2.normalize()?;
        return Ok(t1.dot(t2) > 1.0 - conf);
    }
    Ok(false)
}
