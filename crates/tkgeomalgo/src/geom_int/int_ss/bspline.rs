//! The polygonal B-splines of the points of a walking line, and the U translation of a
//! 2d curve into the period of its surface.
use super::GeomIntIntSS;
use crate::int_patch::{IntPatchPointLine, IntPatchWLine};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, arrays};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomSurface};
use rustycad_tkmath::gp::{Point2, Point3, Vector2};

/// The knots `0..nbpnt - 1` and the multiplicities `2, 1, ..., 1, 2` of a polygon of
/// `nbpnt` points.
fn polygon_knots(nbpnt: i32) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut knots: Array1<'static, f64> = arrays::one_based(nbpnt)?;
    let mut mults: Array1<'static, i32> = arrays::one_based(nbpnt)?;
    for i in 1..=nbpnt {
        arrays::put(&mut knots, i, f64::from(i - 1))?;
        arrays::put(&mut mults, i, 1)?;
    }
    arrays::put(&mut mults, 1, 2)?;
    arrays::put(&mut mults, nbpnt, 2)?;
    Ok((knots, mults))
}

/// `AdjustUPeriodic(aS, aC2D)`: a 2d curve on a U-periodic surface is translated by
/// periods so that its point at 0.467 of its range is within the U bounds.
pub(super) fn adjust_u_periodic(
    s: &Handle<dyn GeomSurface>,
    c2d: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let Some(c) = c2d.get() else {
        return Ok(());
    };
    if !s.is_u_periodic() {
        return Ok(());
    }
    let eps: f64 = precision::p_confusion();
    let epsilon10: f64 = epsilon(10.0);
    let (umin, umax, _, _) = s.bounds();
    let period: f64 = s.u_period()?;
    let (t1, t2) = (c.first_parameter(), c.last_parameter());
    let tx: f64 = t1 + 0.467 * (t2 - t1);
    let px: Point2 = c.value(tx)?;
    let mut ux: f64 = px.x();
    if ux.abs() < epsilon10 {
        ux = 0.0;
    }
    if (ux - period).abs() < epsilon10 {
        ux = period;
    }
    let mut du: f64 = 0.0;
    while ux < (umin - eps) {
        ux += period;
        du += period;
    }
    while ux > (umax + eps) {
        ux -= period;
        du -= period;
    }
    if du != 0.0 {
        c.translate(Vector2::new(du, 0.0))?;
    }
    Ok(())
}

impl GeomIntIntSS {
    /// `MakeBSpline(WL, ideb, ifin)`: the polygon of degree 1 through the points
    /// `ideb..=ifin` of the line, the knots being their ranks from 0;
    /// `GeomIntIntSS::make_bspline(&wl, 1, 10)?`.
    pub fn make_bspline(
        wl: &Handle<IntPatchWLine>,
        ideb: i32,
        ifin: i32,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let nbpnt: i32 = ifin - ideb + 1;
        let mut poles: Array1<'static, Point3> = arrays::one_based(nbpnt)?;
        for i in 1..=nbpnt {
            arrays::put(&mut poles, i, wl.point(ideb + i - 1)?.value())?;
        }
        let (knots, mults) = polygon_knots(nbpnt)?;
        Ok(Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast())
    }

    /// `MakeBSpline2d(theWLine, ideb, ifin, onFirst)`: the polygon of degree 1 through
    /// the parameters on one surface of the points `ideb..=ifin`;
    /// `GeomIntIntSS::make_bspline2d(&wl, 1, 10, true)?`.
    pub fn make_bspline2d(
        wline: &Handle<IntPatchWLine>,
        ideb: i32,
        ifin: i32,
        on_first: bool,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let nbpnt: i32 = ifin - ideb + 1;
        let mut poles: Array1<'static, Point2> = arrays::one_based(nbpnt)?;
        for i in 1..=nbpnt {
            let (u, v) = wline.point(ideb + i - 1)?.parameters_on_surface(on_first);
            arrays::put(&mut poles, i, Point2::new(u, v))?;
        }
        let (knots, mults) = polygon_knots(nbpnt)?;
        Ok(Handle::new(Geom2dBSplineCurve::new(
            &poles, &knots, &mults, 1, false,
        )?))
    }
}
