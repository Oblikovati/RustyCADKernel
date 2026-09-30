// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (isIsoLine,
// buildC3dOnIsoLine) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The isoline members of GeomLib: recognizing a pcurve that is an isoline and
//! building its 3D curve from the surface isoline.
use super::live;
use crate::geom_convert;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{self, Direction2, Point2, Vector2};

// The number of segments of the deviation check of buildC3dOnIsoLine.
const THE_ISOLINE_CHECK_SEGMENTS: i32 = 23;

/// The isoline a pcurve lies on: whether it is a U isoline (U constant), its
/// constant parameter, and whether it runs forward along the other parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IsoLine {
    /// `theIsU`: the line is U = const.
    pub is_u: bool,
    /// `theParam`: the constant parameter.
    pub param: f64,
    /// `theIsForward`: the line runs in the increasing direction.
    pub is_forward: bool,
}

/// `GeomLib::isIsoLine(theC2D, theIsU, theParam, theIsForward)`: the isoline of a 2D
/// line, or of a degree-1 two-pole B-spline or Bezier curve (natural
/// parameterization) whose direction is parallel to an axis within
/// Precision::Angular(); None otherwise (the native false, which leaves the outputs
/// unwritten); `geom_lib::is_iso_line(&pcurve)?`.
pub fn is_iso_line(c2d: &Handle<dyn Adaptor2dCurve2d>) -> Result<Option<IsoLine>, Failure> {
    let curve: &dyn Adaptor2dCurve2d = live(c2d, "GeomLib::isIsoLine")?;
    let Some((loc_2d, dir_2d)) = line_of(curve)? else {
        return Ok(None);
    };
    // Check line to be vertical or horizontal.
    if dir_2d.is_parallel(gp::dx2d(), precision::angular()) {
        // Horizontal line. V = const.
        return Ok(Some(IsoLine {
            is_u: false,
            param: loc_2d.y(),
            is_forward: dir_2d.dot(gp::dx2d()) > 0.0,
        }));
    }
    if dir_2d.is_parallel(gp::dy2d(), precision::angular()) {
        // Vertical line. U = const.
        return Ok(Some(IsoLine {
            is_u: true,
            param: loc_2d.x(),
            is_forward: dir_2d.dot(gp::dy2d()) > 0.0,
        }));
    }
    Ok(None)
}

// The location and direction of a line-like curve, None for another type, a spline
// that is not a two-pole segment, or a degenerated one.
fn line_of(curve: &dyn Adaptor2dCurve2d) -> Result<Option<(Point2, Direction2)>, Failure> {
    let (degree, nb_poles, p1, p2) = match curve.get_type()? {
        CurveType::Line => {
            let lin = curve.line()?;
            return Ok(Some((lin.location(), lin.direction())));
        }
        CurveType::BSplineCurve => {
            let bspline: Handle<Geom2dBSplineCurve> = curve.bspline()?;
            let b: &Geom2dBSplineCurve = live(&bspline, "GeomLib::isIsoLine")?;
            (
                b.degree(),
                b.nb_poles(),
                b.pole(1)?,
                b.pole(2.min(b.nb_poles()))?,
            )
        }
        CurveType::BezierCurve => {
            let bezier: Handle<Geom2dBezierCurve> = curve.bezier()?;
            let b: &Geom2dBezierCurve = live(&bezier, "GeomLib::isIsoLine")?;
            (
                b.degree(),
                b.nb_poles(),
                b.pole(1)?,
                b.pole(2.min(b.nb_poles()))?,
            )
        }
        _ => return Ok(None),
    };
    if degree != 1 || nb_poles != 2 {
        return Ok(None); // Not a line or uneven parameterization.
    }
    // Vector should be non-degenerated.
    let vec_2d = Vector2::between(p1, p2);
    if vec_2d.square_magnitude() < precision::confusion() * precision::confusion() {
        return Ok(None); // Degenerated spline.
    }
    Ok(Some((p1, Direction2::from_vector(vec_2d)?)))
}

/// `GeomLib::buildC3dOnIsoLine(theC2D, theSurf, theFirst, theLast, theTolerance,
/// theIsU, theParam, theIsForward)`: the surface isoline over the pcurve's range,
/// converted to a B-spline (quasi-angular), reversed when backward and
/// reparametrized as the pcurve; null when the surface is not a GeomAdaptor_Surface
/// or is a sphere, when the range misses the surface bounds (within the tolerance)
/// or is null, or when the deviation from the pcurve at 24 samples of
/// `[theFirst, theLast]` exceeds `tolerance` (as on isolines of a sphere);
/// `geom_lib::build_c3d_on_iso_line(&pcurve, &surface, 0.0, 1.0, 1e-7, iso)?`.
pub fn build_c3d_on_iso_line(
    c2d: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
    (first, last): (f64, f64),
    tolerance: f64,
    iso: IsoLine,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    // Convert adapter to the appropriate type.
    let adapter: Handle<GeomAdaptorSurface> = surf.cast();
    let Some(geom_adapter) = adapter.get() else {
        return Ok(Handle::default());
    };
    if live(surf, "GeomLib::buildC3dOnIsoLine")?.get_type()? == SurfaceType::Sphere {
        return Ok(Handle::default());
    }
    let curve_2d: &dyn Adaptor2dCurve2d = live(c2d, "GeomLib::buildC3dOnIsoLine")?;
    let a_f2d: Point2 = curve_2d.eval_d0(curve_2d.first_parameter()?)?;
    let a_l2d: Point2 = curve_2d.eval_d0(curve_2d.last_parameter()?)?;
    let range: [f64; 2] = if iso.is_u {
        [a_f2d.y(), a_l2d.y()]
    } else {
        [a_f2d.x(), a_l2d.x()]
    };
    let c3d: Handle<dyn GeomCurve> = extracted_iso(&geom_adapter.surface(), range, tolerance, iso)?;
    if c3d.is_null() {
        return Ok(c3d);
    }
    // Convert arbitrary curve type to the b-spline.
    let curve_3d: Handle<GeomBSplineCurve> =
        geom_convert::curve_to_bspline_curve(&c3d, ParameterisationType::QuasiAngular)?;
    let bspline: &GeomBSplineCurve = live(&curve_3d, "GeomLib::buildC3dOnIsoLine")?;
    if !iso.is_forward {
        bspline.reverse()?;
    }
    // Rebuild the parameterization of the 3d curve to match the 2d curve.
    let mut knots: Array1<'static, f64> = bspline.knots();
    bsplclib::reparametrize(
        curve_2d.first_parameter()?,
        curve_2d.last_parameter()?,
        &mut knots,
    )?;
    bspline.set_knots(&knots)?;
    // Target tolerance not reached: this happens for isolines on the sphere, which
    // the general-purpose approach handles better.
    if iso_deviation(
        curve_2d,
        bspline,
        live(surf, "GeomLib::buildC3dOnIsoLine")?,
        [first, last],
    )? > tolerance
    {
        return Ok(Handle::default());
    }
    Ok(curve_3d.cast())
}

// The isoline of the surface over `range` (sorted), or a null curve when the range
// misses the surface bounds or is null: trimmed when the surface is bounded in that
// direction, taken on a surface trimmed to the range when it is infinite.
fn extracted_iso(
    surface: &Handle<dyn GeomSurface>,
    [a, b]: [f64; 2],
    tolerance: f64,
    iso: IsoLine,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let s: &dyn GeomSurface = live(surface, "GeomLib::buildC3dOnIsoLine")?;
    let (u1, u2, v1, v2) = s.bounds();
    let (lower, upper): (f64, f64) = if iso.is_u { (v1, v2) } else { (u1, u2) };
    let (mut p1, mut p2): (f64, f64) = (std_min(a, b), std_max(a, b));
    if p2 < lower - tolerance || p1 > upper + tolerance {
        return Ok(Handle::default());
    }
    if precision::is_infinite(lower) || precision::is_infinite(upper) {
        if (p2 - p1).abs() < precision::p_confusion() {
            return Ok(Handle::default());
        }
        let (u, v) = if iso.is_u {
            ((u1, u2), (p1, p2))
        } else {
            ((p1, p2), (v1, v2))
        };
        let trimmed = GeomRectangularTrimmedSurface::new(surface, u, v, (true, true))?;
        let trimmed: Handle<dyn GeomSurface> = Handle::new(trimmed).cast();
        let t: &dyn GeomSurface = live(&trimmed, "GeomLib::buildC3dOnIsoLine")?;
        return if iso.is_u {
            t.u_iso(iso.param)
        } else {
            t.v_iso(iso.param)
        };
    }
    p1 = std_max(p1, lower);
    p2 = std_min(p2, upper);
    if (p2 - p1).abs() < precision::p_confusion() {
        return Ok(Handle::default());
    }
    let c3d: Handle<dyn GeomCurve> = if iso.is_u {
        s.u_iso(iso.param)?
    } else {
        s.v_iso(iso.param)?
    };
    Ok(Handle::new(GeomTrimmedCurve::new(&c3d, p1, p2, true, true)?).cast())
}

// The greatest distance between the 3D curve and the surface point of the pcurve at
// 24 regular parameters of [first, last].
fn iso_deviation(
    c2d: &dyn Adaptor2dCurve2d,
    curve_3d: &GeomBSplineCurve,
    surf: &dyn Adaptor3dSurface,
    [par_f, par_l]: [f64; 2],
) -> Result<f64, Failure> {
    let mut error_3d: f64 = 0.0;
    for idx in 0..=THE_ISOLINE_CHECK_SEGMENTS {
        let par: f64 =
            par_f + ((par_l - par_f) * f64::from(idx)) / f64::from(THE_ISOLINE_CHECK_SEGMENTS);
        let pnt_2d: Point2 = c2d.eval_d0(par)?;
        let pnt_c3d = curve_3d.eval_d0(par)?;
        let pnt_c2d = surf.eval_d0(pnt_2d.x(), pnt_2d.y())?;
        error_3d = std_max(pnt_c3d.square_distance(&pnt_c2d), error_3d);
    }
    Ok(error_3d.sqrt())
}
