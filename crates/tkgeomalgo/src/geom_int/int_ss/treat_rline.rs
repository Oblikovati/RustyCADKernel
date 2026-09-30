//! `TreatRLine`: the curves of a restriction line, the 3d curve as the approximation of
//! the curve on the surface of its arc (same parameter).
use super::GeomIntIntSS;
use crate::int_patch::IntPatchRLine;
use crate::int_start::TheArc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::make_curve as make_curve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, make_surface};
use rustycad_tkgeombase::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

/// The in-out curves of `TreatRLine`: the 3d curve, the 2d curves on both surfaces and
/// the reached tolerance (the 2d curves already set are kept).
///
/// ```
/// use rustycad_tkgeomalgo::geom_int::GeomIntRLineCurves;
/// let curves = GeomIntRLineCurves::default();
/// assert!(curves.c3d.is_null());
/// ```
#[derive(Clone, Default)]
pub struct GeomIntRLineCurves {
    /// `theC3d`.
    pub c3d: Handle<dyn GeomCurve>,
    /// `theC2d1`.
    pub c2d1: Handle<dyn Geom2dCurve>,
    /// `theC2d2`.
    pub c2d2: Handle<dyn Geom2dCurve>,
    /// `theTolReached` (left unchanged when no 3d curve is made).
    pub tol_reached: f64,
}

/// `isDegenerated(theGAHS, theAHC2d, theFirstPar, theLastPar)`: whether the arc is a
/// degenerated edge (its ends and middle on the same 3d point within Confusion).
fn is_degenerated(
    gahs: &GeomAdaptorSurface,
    ahc2d: &dyn Adaptor2dCurve2d,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    let sq_tol: f64 = precision::confusion() * precision::confusion();
    let image = |t: f64| -> Result<Point3, Failure> {
        let p2d: Point2 = ahc2d.d0(t)?;
        gahs.d0(p2d.x(), p2d.y())
    };
    let p1: Point3 = image(first)?;
    let p2: Point3 = image(last)?;
    if p1.square_distance(&p2) > sq_tol {
        return Ok(false);
    }
    let pm: Point3 = image(0.5 * (first + last))?;
    Ok(p1.square_distance(&pm) <= sq_tol)
}

/// The arc of the restriction line on its surface: the arc, its trimmed 2d curve and
/// its range clamped to the curve.
fn trimmed_arc(
    arc: &TheArc,
    (tf, tl): (f64, f64),
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let Some(adaptor) = arc.get() else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "GeomInt_IntSS::TreatRLine() - the arc is null; expected a 2d curve",
        ));
    };
    let c2d: Handle<dyn Geom2dCurve> = make_curve2d(adaptor)?;
    let tf: f64 = std_max(tf, c2d.first_parameter());
    let tl: f64 = std_min(tl, c2d.last_parameter());
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&c2d, tf, tl, true, true)?).cast();
    Ok((trimmed, tf, tl))
}

impl GeomIntIntSS {
    /// `TreatRLine(theRL, theHS1, theHS2, theC3d, theC2d1, theC2d2, theTolReached)`: the
    /// 3d curve of a restriction line (the approximation, of degree 8 and C1, of its arc
    /// on its surface) and its 2d curves (the trimmed arc and the projection on the
    /// other surface); nothing for a degenerated arc or a line on no arc;
    /// `GeomIntIntSS::treat_rline(&rl, &hs1, &hs2, &mut curves)?`.
    pub fn treat_rline(
        rl: &Handle<IntPatchRLine>,
        hs1: &Handle<GeomAdaptorSurface>,
        hs2: &Handle<GeomAdaptorSurface>,
        out: &mut GeomIntRLineCurves,
    ) -> Result<(), Failure> {
        // The 2d curve is assumed to be a 2d line (rectangular surface domain).
        let (gahs, arc, tf, tl) = if rl.is_arc_on_s1() {
            let arc: TheArc = rl.arc_on_s1();
            let (c2d, tf, tl) = trimmed_arc(&arc, rl.param_on_s1())?;
            out.c2d1 = c2d;
            (hs1, arc, tf, tl)
        } else if rl.is_arc_on_s2() {
            let arc: TheArc = rl.arc_on_s2();
            let (c2d, tf, tl) = trimmed_arc(&arc, rl.param_on_s2())?;
            out.c2d2 = c2d;
            (hs2, arc, tf, tl)
        } else {
            return Ok(());
        };
        // The restriction line can be a degenerated edge: no curve then.
        if is_degenerated(gahs, &*arc, tf, tl)? {
            return Ok(());
        }
        // Same parameter: the 3d curve is the approximation of the curve on surface.
        let surface: Handle<dyn Adaptor3dSurface> = gahs.cast();
        let options = CurveOnSurfaceOptions {
            max_segments: 1000,
            max_degree: 8,
            continuity: Shape::C1,
            only_3d: true,
            only_2d: false,
        };
        let app = CurveOnSurface::new(&arc, &surface, tf, tl, precision::confusion(), options)?;
        if !app.has_result() {
            return Ok(());
        }
        out.c3d = app.curve_3d().cast();
        out.tol_reached = app.max_error_3d();
        let mut tol: f64 = precision::confusion();
        if rl.is_arc_on_s1() {
            let s: Handle<dyn GeomSurface> = make_surface(&**hs2, true)?;
            Self::build_pcurves(tf, tl, &mut tol, &s, &out.c3d, &mut out.c2d2)?;
        }
        if rl.is_arc_on_s2() {
            let s: Handle<dyn GeomSurface> = make_surface(&**hs1, true)?;
            Self::build_pcurves(tf, tl, &mut tol, &s, &out.c3d, &mut out.c2d1)?;
        }
        out.tol_reached = std_max(out.tol_reached, tol);
        Ok(())
    }
}
