//! The parameters where the 2d curves of an intersection line cross the rectangular
//! boundaries of the surfaces.
use super::GeomIntIntSS;
use crate::geom2d_int::GInter;
use crate::int_res2d::IntersectionSegment;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_small;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl::sort_by;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `GetSegmentBoundary(theSegm, theCurve, theArrayOfParameters)`: the parameters on the
/// first curve of the ends of the segment (the curve bounds for a missing end).
fn get_segment_boundary(
    segm: &IntersectionSegment,
    curve: &Handle<dyn Geom2dCurve>,
    parameters: &mut Vec<f64>,
) -> Result<(), Failure> {
    let mut u1: f64 = curve.first_parameter();
    let mut u2: f64 = curve.last_parameter();
    if segm.has_first_point() {
        u1 = segm.first_point()?.param_on_first();
    }
    if segm.has_last_point() {
        u2 = segm.last_point()?.param_on_first();
    }
    parameters.push(u1);
    parameters.push(u2);
    Ok(())
}

/// `IntersectCurveAndBoundary(theC2d, theArrBounds, theNumberOfCurves, theTol,
/// theArrayOfParameters)`: the parameters on `c2d` of its intersection points and
/// segments with each boundary curve.
fn intersect_curve_and_boundary(
    c2d: &Handle<dyn Geom2dCurve>,
    bounds: &[Handle<dyn Geom2dCurve>; 4],
    tol: f64,
    parameters: &mut Vec<f64>,
) -> Result<(), Failure> {
    if c2d.is_null() {
        return Ok(());
    }
    let ac1 = Geom2dAdaptorCurve::with_curve(c2d)?;
    for bound in bounds.iter().filter(|b| !b.is_null()) {
        let ac2 = Geom2dAdaptorCurve::with_curve(bound)?;
        let int_cc2d = GInter::between(&ac1, &ac2, tol, tol)?;
        if !int_cc2d.is_done() || int_cc2d.is_empty()? {
            continue;
        }
        for pnt_id in 1..=int_cc2d.nb_points()? {
            parameters.push(int_cc2d.point(pnt_id)?.param_on_first());
        }
        for segm_id in 1..=int_cc2d.nb_segments()? {
            get_segment_boundary(&int_cc2d.segment(segm_id)?, c2d, parameters)?;
        }
    }
    Ok(())
}

/// A boundary line from `origin` along `direction`, trimmed to `[0, delta]` unless the
/// extent is infinite.
fn bound_line(
    origin: Point2,
    direction: Direction2,
    delta: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(origin, direction)).cast();
    if precision::is_infinite(delta) {
        return Ok(line);
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, 0.0, delta, true, true)?).cast())
}

/// The four boundaries of a box: `[0]` U = Ufirst, `[1]` U = Ulast, `[2]` V = Vfirst,
/// `[3]` V = Vlast (null for an infinite bound or a flat box).
fn box_boundaries(bound: &BoundingBox2) -> Result<[Handle<dyn Geom2dCurve>; 4], Failure> {
    let [uf, vf, ul, vl] = bound.get()?;
    let mut curves: [Handle<dyn Geom2dCurve>; 4] = Default::default();
    let (dir_x, dir_y) = (Direction2::X, Direction2::Y);
    let delta_v: f64 = vl - vf;
    if delta_v.abs() > real_small() {
        if !precision::is_infinite(uf) {
            curves[0] = bound_line(Point2::new(uf, vf), dir_y, delta_v)?;
        }
        if !precision::is_infinite(ul) {
            curves[1] = bound_line(Point2::new(ul, vf), dir_y, delta_v)?;
        }
    }
    let delta_u: f64 = ul - uf;
    if delta_u.abs() > real_small() {
        if !precision::is_infinite(vf) {
            curves[2] = bound_line(Point2::new(uf, vf), dir_x, delta_u)?;
        }
        if !precision::is_infinite(vl) {
            curves[3] = bound_line(Point2::new(uf, vl), dir_x, delta_u)?;
        }
    }
    Ok(curves)
}

impl GeomIntIntSS {
    /// `TrimILineOnSurfBoundaries(theC2d1, theC2d2, theBound1, theBound2,
    /// theArrayOfParameters)`: appends the parameters where the 2d curves cross the
    /// boundaries of the boxes (with the tolerance `10 * Precision::Confusion()`), then
    /// sorts the parameters (libstdc++ `std::sort`);
    /// `GeomIntIntSS::trim_iline_on_surf_boundaries(&c1, &c2, &b1, &b2, &mut params)?`.
    pub fn trim_iline_on_surf_boundaries(
        c2d1: &Handle<dyn Geom2dCurve>,
        c2d2: &Handle<dyn Geom2dCurve>,
        bound1: &BoundingBox2,
        bound2: &BoundingBox2,
        parameters: &mut Vec<f64>,
    ) -> Result<(), Failure> {
        let cur_s1_bounds = box_boundaries(bound1)?;
        let cur_s2_bounds = box_boundaries(bound2)?;
        let int_tol: f64 = 10.0 * precision::confusion();
        intersect_curve_and_boundary(c2d1, &cur_s1_bounds, int_tol, parameters)?;
        intersect_curve_and_boundary(c2d2, &cur_s2_bounds, int_tol, parameters)?;
        sort_by(parameters, |a, b| a < b);
        Ok(())
    }
}
