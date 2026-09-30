// Rust translation of ModelingData/TKGeomBase/GeomProjLib/GeomProjLib.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomProjLib: the projection of a Geom curve on a Geom surface.
//!
//! The six `GeomProjLib::Curve2d` overloads give the 2D curve of a 3D curve lying
//! on a surface ([`curve2d_range_bounds_tolerance`] and the forms that default its
//! range, bounds or tolerance), [`project`] builds the 3D curve of the normal
//! projection on a surface and [`project_on_plane`] the projection on a plane along
//! a direction. The functions build on ProjLib (`ProjectedCurve`,
//! `CompProjectedCurve`, `ProjectOnPlane`) and `Approx_CurveOnSurface`.
//!
//! The native in-out `Tolerance` reference is an input and a second returned value.
//! A projection that fails returns a null handle, as natively; native exceptions are
//! `Failure` errors.
mod project;

pub use project::{project, project_on_plane};

use crate::proj_lib::ProjectedCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
    Geom2dTrimmedCurve,
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::geom_abs::CurveType;

/// A 2D curve with the tolerance reached (the native in-out `Tolerance`).
pub type Curve2dWithTolerance = (Handle<dyn Geom2dCurve>, f64);

/// `GeomProjLib::Curve2d(C, First, Last, S, UFirst, ULast, VFirst, VLast, Tolerance)`:
/// the 2D curve of `curve` over `[first, last]` lying on `surface` restricted to
/// `bounds`, through `ProjLib_ProjectedCurve` with `max(PConfusion, tolerance)`.
/// Returns the curve (null when the projection failed) and the tolerance reached by
/// the approximation (the raised input tolerance when the result kind is not a
/// curve kind). A trimmed `curve` gives a trimmed result, clamped to a non-periodic
/// result's range.
///
/// ```ignore
/// let (c2d, reached) =
///     curve2d_range_bounds_tolerance(&c, (0.0, 1.0), &s, (0.0, 1.0, 0.0, 1.0), 1e-7)?;
/// ```
pub fn curve2d_range_bounds_tolerance(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
    tolerance: f64,
) -> Result<Curve2dWithTolerance, Failure> {
    let tolerance: f64 = std_max(precision::p_confusion(), tolerance);
    let adapted_curve = GeomAdaptorCurve::with_range(curve, range.0, range.1)?;
    let adapted_surface = GeomAdaptorSurface::with_bounds(surface, bounds)?;
    let hs: Handle<dyn Adaptor3dSurface> = Handle::new(adapted_surface).cast();
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(adapted_curve).cast();
    let projection: ProjectedCurve = ProjectedCurve::with_tolerance(&hs, &hc, tolerance)?;
    let Some(g2d) = typed_curve2d(&projection)? else {
        return Ok((Handle::default(), tolerance));
    };
    if g2d.is_null() {
        return Ok((g2d, projection.get_tolerance()));
    }
    let g2d: Handle<dyn Geom2dCurve> = trim_like(curve, g2d)?;
    Ok((g2d, projection.get_tolerance()))
}

/// `GeomProjLib::Curve2d(C, First, Last, S, Tolerance)`: over the whole surface
/// bounds; `curve2d_range_tolerance(&c, (0.0, 1.0), &s, 1e-7)?`.
pub fn curve2d_range_tolerance(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<Curve2dWithTolerance, Failure> {
    let bounds: SurfaceBounds = live_surface(surface)?.bounds();
    curve2d_range_bounds_tolerance(curve, range, surface, bounds, tolerance)
}

/// `GeomProjLib::Curve2d(C, First, Last, S)`: with the tolerance PConfusion;
/// `curve2d_range(&c, (0.0, 1.0), &s)?`.
pub fn curve2d_range(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    surface: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(curve2d_range_tolerance(curve, range, surface, precision::p_confusion())?.0)
}

/// `GeomProjLib::Curve2d(C, S)`: over the curve's own range, with the tolerance
/// PConfusion; `curve2d(&c, &s)?`.
pub fn curve2d(
    curve: &Handle<dyn GeomCurve>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let range: (f64, f64) = curve_range(curve)?;
    Ok(curve2d_range_tolerance(curve, range, surface, precision::p_confusion())?.0)
}

/// `GeomProjLib::Curve2d(C, S, UDeb, UFin, VDeb, VFin)`: over the curve's own range
/// on the surface restricted to `bounds` (which may extend it a little), with the
/// tolerance PConfusion; `curve2d_bounds(&c, &s, (0.0, 1.0, 0.0, 1.0))?`.
pub fn curve2d_bounds(
    curve: &Handle<dyn GeomCurve>,
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let tolerance: f64 = precision::p_confusion();
    Ok(curve2d_bounds_tolerance(curve, surface, bounds, tolerance)?.0)
}

/// `GeomProjLib::Curve2d(C, S, UDeb, UFin, VDeb, VFin, Tolerance)`: over the curve's
/// own range; `curve2d_bounds_tolerance(&c, &s, (0.0, 1.0, 0.0, 1.0), 1e-7)?`.
pub fn curve2d_bounds_tolerance(
    curve: &Handle<dyn GeomCurve>,
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
    tolerance: f64,
) -> Result<Curve2dWithTolerance, Failure> {
    let range: (f64, f64) = curve_range(curve)?;
    curve2d_range_bounds_tolerance(curve, range, surface, bounds, tolerance)
}

// The native `C->FirstParameter()` and `C->LastParameter()` dereference the handle.
fn curve_range(curve: &Handle<dyn GeomCurve>) -> Result<(f64, f64), Failure> {
    let c: &dyn GeomCurve = curve.get().ok_or_else(|| null_input("the curve C"))?;
    Ok((c.first_parameter(), c.last_parameter()))
}

// The native `S->Bounds(...)` dereferences the handle.
fn live_surface(surface: &Handle<dyn GeomSurface>) -> Result<&dyn GeomSurface, Failure> {
    surface.get().ok_or_else(|| null_input("the surface S"))
}

pub(super) fn null_input(what: &str) -> Failure {
    crate::native_failure::failure(
        "Standard_NullObject",
        &format!("GeomProjLib: {what} is a null handle; expected a curve and a surface"),
    )
}

// The Geom2d curve of the projection kind; None for the other kinds (the native
// `default: return G2dC;`). The Bezier and B-spline results are the stored handles.
fn typed_curve2d(projection: &ProjectedCurve) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let curve: Handle<dyn Geom2dCurve> = match projection.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(projection.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(projection.circle()?)).cast(),
        CurveType::Ellipse => {
            Handle::new(Geom2dEllipse::from_elips2d(projection.ellipse()?)).cast()
        }
        CurveType::Parabola => {
            Handle::new(Geom2dParabola::from_parab2d(projection.parabola()?)).cast()
        }
        CurveType::Hyperbola => {
            Handle::new(Geom2dHyperbola::from_hypr2d(projection.hyperbola()?)).cast()
        }
        CurveType::BezierCurve => projection.bezier()?.cast(),
        CurveType::BSplineCurve => projection.bspline()?.cast(),
        _ => return Ok(None),
    };
    Ok(Some(curve))
}

// The trim parameters of a Geom_TrimmedCurve (the native `IsKind` and `DownCast`).
pub(super) fn trimmed_range(curve: &Handle<dyn GeomCurve>) -> Option<(f64, f64)> {
    let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
    let t: &GeomTrimmedCurve = trimmed.get()?;
    Some((t.first_parameter(), t.last_parameter()))
}

// A trimmed 3D curve gives a Geom2d_TrimmedCurve over its trim parameters, clamped
// to the range of a non-periodic 2D curve.
fn trim_like(
    curve: &Handle<dyn GeomCurve>,
    g2d: Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let Some((mut u1, mut u2)) = trimmed_range(curve) else {
        return Ok(g2d);
    };
    let c2d: &dyn Geom2dCurve = g2d.get().ok_or_else(|| null_input("the 2D result"))?;
    if !c2d.is_periodic() {
        u1 = std_max(u1, c2d.first_parameter());
        u2 = std_min(u2, c2d.last_parameter());
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(&g2d, u1, u2, true, true)?).cast())
}
