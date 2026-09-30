//! `BuildPCurves`: the 2d curve of a 3d curve on a surface, by projection (or as a
//! segment between the projections of the ends on a range too short for the
//! projection), moved into the U period of the surface.
use super::GeomIntIntSS;
use crate::geom_int::adjust_periodic::adjust_periodic;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, arrays};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtPS};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{self, Point2, Point3, Vector2};

/// `ParametersOfNearestPointOnSurface(theExtr, theU, theV)`: the parameters of the
/// nearest extremum (the first of equal distances), None when there is none.
fn parameters_of_nearest_point_on_surface(extr: &ExtPS<'_>) -> Result<Option<(f64, f64)>, Failure> {
    if !extr.is_done() || extr.nb_ext()? == 0 {
        return Ok(None);
    }
    let mut index: i32 = 1;
    let mut min_sq_dist: f64 = extr.square_distance(index)?;
    for i in 2..=extr.nb_ext()? {
        let sqd: f64 = extr.square_distance(i)?;
        if sqd < min_sq_dist {
            min_sq_dist = sqd;
            index = i;
        }
    }
    Ok(Some(extr.point(index)?.parameter()))
}

/// The projection of `curve` over `first..last` on the bounded surface: the tolerance
/// grows by `Precision::Confusion()` for a second projection on the whole surface when
/// the first fails (a circle through the pole of a sphere). A B-spline result whose
/// ends differ from the range is reparametrized onto it (as `Geom2d_TrimmedCurve`
/// checks them).
fn projected(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
    tol: &mut f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (mut c2d, reached) =
        geom_proj_lib::curve2d_range_bounds_tolerance(curve, (first, last), surface, bounds, *tol)?;
    *tol = reached;
    if c2d.is_null() {
        *tol += precision::confusion();
        (c2d, *tol) = geom_proj_lib::curve2d_range_tolerance(curve, (first, last), surface, *tol)?;
    }
    let Some(c) = c2d.get() else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "GeomInt_IntSS::BuildPCurves() - both projections failed; expected a 2d curve",
        ));
    };
    let bspl: Handle<Geom2dBSplineCurve> = c2d.cast();
    if let Some(b) = bspl.get() {
        let pconf: f64 = precision::p_confusion();
        if c.first_parameter() - first > pconf || last - c.last_parameter() > pconf {
            let mut knots: Array1<'static, f64> = b.knots();
            bsplclib::reparametrize(first, last, &mut knots)?;
            b.set_knots(&knots)?;
        }
    }
    Ok(c2d)
}

/// The segment between the projections of the ends of `curve` on a range shorter than
/// the shrinkage of `ProjLib_Function` (2e-9); `tol` grows to the distance at the middle.
fn short_segment(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    surface: &Handle<dyn GeomSurface>,
    (umin, umax, vmin, vmax): SurfaceBounds,
    tol: &mut f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let an_as = GeomAdaptorSurface::default();
    an_as.load(surface)?;
    let mut extr: ExtPS<'_> = ExtPS::new();
    let p3d1: Point3 = curve.value(first)?;
    let p3d2: Point3 = curve.value(last)?;
    extr.set_algo(ExtAlgo::Grad);
    let conf: f64 = precision::confusion();
    extr.initialize(&an_as, (umin, umax), (vmin, vmax), (conf, conf))?;
    extr.perform(p3d1)?;
    let Some((u1, v1)) = parameters_of_nearest_point_on_surface(&extr)? else {
        return Ok(Handle::default());
    };
    let p2d1 = Point2::new(u1, v1);
    extr.perform(p3d2)?;
    let Some((u2, v2)) = parameters_of_nearest_point_on_surface(&extr)? else {
        return Ok(Handle::default());
    };
    let p2d2 = Point2::new(u2, v2);
    if p2d1.distance(p2d2) <= gp::resolution() {
        return Ok(Handle::default());
    }
    let mut poles: Array1<'static, Point2> = arrays::one_based(2)?;
    let mut knots: Array1<'static, f64> = arrays::one_based(2)?;
    let mut mults: Array1<'static, i32> = arrays::one_based(2)?;
    (poles[1], poles[2]) = (p2d1, p2d2);
    (knots[1], knots[2]) = (first, last);
    (mults[1], mults[2]) = (2, 2);
    let c2d: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast();
    // Check the same parameter at the middle point.
    let pmid: Point3 = curve.value(0.5 * (first + last))?;
    let pmidcurve2d = Point2::from((p2d1.xy() + p2d2.xy()) * 0.5);
    let pc: Point3 = an_as.value(pmidcurve2d.x(), pmidcurve2d.y())?;
    let dist: f64 = pmid.distance(&pc);
    *tol = std_max(dist, *tol);
    Ok(c2d)
}

impl GeomIntIntSS {
    /// `BuildPCurves(theFirst, theLast, theUmin, theUmax, theVmin, theVmax, theTol,
    /// theSurface, theCurve, theCurve2d)`: when `curve2d` is null and the surface is
    /// not, the 2d curve of `curve` over `first..last` on the surface restricted to the
    /// bounds; `tol` gets the reached tolerance. The 2d curve is translated into the U
    /// period of the surface; `GeomIntIntSS::build_pcurves_bounds(0.0, 1.0, bounds, &mut tol, &s, &c, &mut c2d)?`.
    pub fn build_pcurves_bounds(
        first: f64,
        last: f64,
        bounds: SurfaceBounds,
        tol: &mut f64,
        surface: &Handle<dyn GeomSurface>,
        curve: &Handle<dyn GeomCurve>,
        curve2d: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let Some(s) = surface.get() else {
            return Ok(());
        };
        if !curve2d.is_null() {
            return Ok(());
        }
        // In ProjLib_Function the range of parameters is shrunk by 1.e-09.
        if (last - first) > 2.0e-09 {
            *curve2d = projected(curve, (first, last), surface, bounds, tol)?;
        } else if (last - first) > epsilon(first.abs()) {
            // On [Epsilon(|f|), 2.e-09] the 2d curve can be considered as a segment.
            *curve2d = short_segment(curve, (first, last), surface, bounds, tol)?;
        }
        if s.is_u_periodic() && !curve2d.is_null() {
            Self::recadre_u(s, curve2d, (first, last), bounds)?;
        }
        Ok(())
    }

    /// The translation of the 2d curve whose middle point is out of the U bounds by the
    /// whole periods of its distance (`GeomInt::AdjustPeriodic` with PConfusion).
    fn recadre_u(
        s: &dyn GeomSurface,
        curve2d: &Handle<dyn Geom2dCurve>,
        (first, last): (f64, f64),
        (umin, umax, _, _): SurfaceBounds,
    ) -> Result<(), Failure> {
        let eps: f64 = precision::p_confusion();
        let period: f64 = s.u_period()?;
        let tm: f64 = 0.5 * (first + last);
        let pm: Point2 = curve2d.value(tm)?;
        let (adjust, _, du) = adjust_periodic(pm.x(), umin, umax, period, eps);
        if adjust {
            curve2d.translate(Vector2::new(du, 0.0))?;
        }
        Ok(())
    }

    /// `BuildPCurves(f, l, Tol, S, C, C2d)`: over the bounds of the surface;
    /// `GeomIntIntSS::build_pcurves(0.0, 1.0, &mut tol, &s, &c, &mut c2d)?`.
    pub fn build_pcurves(
        first: f64,
        last: f64,
        tol: &mut f64,
        surface: &Handle<dyn GeomSurface>,
        curve: &Handle<dyn GeomCurve>,
        curve2d: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let Some(s) = surface.get() else {
            return Ok(());
        };
        if !curve2d.is_null() {
            return Ok(());
        }
        let bounds: SurfaceBounds = s.bounds();
        Self::build_pcurves_bounds(first, last, bounds, tol, surface, curve, curve2d)
    }
}
