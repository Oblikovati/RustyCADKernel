// Rust translation of ChFi3d_InterPlaneEdge, ChFi3d_CircularSpine and ChFi3d_Spine
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guide lines of the corners and the cut of an edge by a plane.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle};
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkgeombase::int_ana::QuadQuadGeo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// `ChFi3d_InterPlaneEdge(Plan, C, W, Sens, tolc)`: the parameter of the cut of the
/// curve by the plane within its range (± `tolc`, periodic parameters brought into
/// the period from `first - tolc`), the smallest one when `sens` else the largest;
/// `None` without a cut; `inter_plane_edge(&plane, &curve, true, 1e-7)?`.
pub fn inter_plane_edge(
    plan: &Handle<dyn Adaptor3dSurface>,
    c: &Handle<dyn Adaptor3dCurve>,
    sens: bool,
    tolc: f64,
) -> Result<Option<f64>, Failure> {
    let (uf, ul) = (c.first_parameter()?, c.last_parameter()?);
    let mut intersection = HInter::new();
    intersection.perform(c, plan)?;
    if !intersection.is_done() {
        return Ok(None);
    }
    let mut w: Option<f64> = None;
    for iip in 1..=intersection.nb_points()? {
        let mut cw = intersection.point(iip)?.w();
        if c.is_periodic()? {
            cw = elclib::in_period(cw, uf - tolc, uf - tolc + c.period()?);
        }
        if !(uf - tolc <= cw && ul + tolc >= cw) {
            continue;
        }
        w = match w {
            None => Some(cw),
            Some(old) if (sens && cw < old) || (!sens && cw > old) => Some(cw),
            kept => kept,
        };
    }
    Ok(w)
}

/// `ChFi3d_CircularSpine(WFirst, WLast, Pdeb, Vdeb, Pfin, Vfin, rad)`: the circle of
/// radius `rad` around the line where the normal planes of both ends meet, and its
/// range `(0, angle)`; `None` when the planes do not meet in a line or the ends turn
/// the same way; `circular_spine(pd, vd, pf, vf, 2.0)?`.
pub fn circular_spine(
    pdeb: Point3,
    vdeb: Vector3,
    pfin: Point3,
    vfin: Vector3,
    rad: f64,
) -> Result<Option<(Handle<GeomCircle>, f64, f64)>, Failure> {
    let pl1 = Plane3::new(pdeb, Direction3::from_vector(vdeb)?)?;
    let pl2 = Plane3::new(pfin, Direction3::from_vector(vfin)?)?;
    let mut lint = QuadQuadGeo::new();
    lint.perform_planes(pl1, pl2, precision::angular(), precision::confusion())?;
    if !lint.is_done() {
        return Ok(None);
    }
    let li = lint.line(1)?;
    let cendeb = elclib::line_value(elclib::line_parameter(li.position(), pdeb), li.position());
    let cenfin = elclib::line_value(elclib::line_parameter(li.position(), pfin), li.position());
    let (vvdeb, vvfin) = (
        Vector3::between(cendeb, pdeb),
        Vector3::between(cenfin, pfin),
    );
    let (dddeb, ddfin) = (
        Direction3::from_vector(vvdeb)?,
        Direction3::from_vector(vvfin)?,
    );
    if vdeb.crossed(vvdeb).dot(vfin.crossed(vvfin)) > 0.0 {
        return Ok(None);
    }
    let circax2 = Axis2::with_x_direction(cendeb, dddeb.crossed(ddfin)?, dddeb)?;
    let ccc = Circle3::new(circax2, rad)?;
    Ok(Some((
        Handle::new(GeomCircle::from_circ(ccc)),
        0.0,
        dddeb.angle(&ddfin),
    )))
}

/// `ChFi3d_Spine(pd, vd, pf, vf, R)`: the cubic Bezier guide line of a corner from the
/// end points and tangents; `vd` and `vf` are scaled to the inner poles as natively;
/// `spine(pd, &mut vd, pf, &mut vf, 2.0)?`.
pub fn spine(
    pd: Point3,
    vd: &mut Vector3,
    pf: Point3,
    vf: &mut Vector3,
    r: f64,
) -> Result<Handle<GeomBezierCurve>, Failure> {
    let fac = 0.5 * ((PI - vd.angle(*vf)?) * 0.5).tan();
    *vd *= fac * r;
    let pole2 = Point3::new(pd.x() + vd.x(), pd.y() + vd.y(), pd.z() + vd.z());
    *vf *= fac * r;
    let pole3 = Point3::new(pf.x() + vf.x(), pf.y() + vf.y(), pf.z() + vf.z());
    let mut poles = [pd, pole2, pole3, pf];
    Ok(Handle::new(GeomBezierCurve::new(&Array1::from_slice(
        &mut poles,
    ))?))
}
