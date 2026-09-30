// Rust translation of IsIntersect and BRepPrimAPI_MakeRevol::CheckValidity of
// BRepPrimAPI_MakeRevol.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The check that no edge of the swept shape crosses the axis of revolution.
use super::make_revol::BRepPrimAPIMakeRevol;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::ExtCC;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Line3};

/// The default tolerances of `Extrema_ExtCC(C1, C2)`.
const EXTCC_TOLERANCE: f64 = 1.0e-10;

/// Whether a circle `curve` keeps farther from the axis than its radius less
/// Precision::Confusion() (then it cannot cross it).
fn circle_misses_axis(curve: &dyn Adaptor3dCurve, axis: &Line3) -> Result<bool, Failure> {
    if curve.get_type()? != CurveType::Circle {
        return Ok(false);
    }
    let circle = curve.circle()?;
    let mut r2 = circle.radius();
    r2 -= precision::confusion();
    r2 *= r2;
    Ok(axis.square_distance(circle.location()) > r2)
}

/// `IsIntersect(theC, theAxe)`: the curve meets the axis (an extremum at a distance not
/// above Precision::Confusion()) strictly inside its range shrunk by the parametric
/// resolution of Precision::Confusion().
fn is_intersect(curve: &GeomAdaptorCurve, axe: Axis1) -> Result<bool, Failure> {
    let axis = Line3::from_axis(axe);
    if circle_misses_axis(curve, &axis)? {
        return Ok(false);
    }
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(axis)).cast();
    let lin = GeomAdaptorCurve::with_curve(&line)?;
    let par_tol = curve.resolution(precision::confusion())?;
    let par_f = curve.first_parameter()? + par_tol;
    let par_l = curve.last_parameter()? - par_tol;
    let mut extrema = ExtCC::with_curves(curve, &lin, EXTCC_TOLERANCE, EXTCC_TOLERANCE)?;
    extrema.perform()?;
    if !extrema.is_done() || extrema.nb_ext()? <= 0 {
        return Ok(false);
    }
    for i in 1..=extrema.nb_ext()? {
        if extrema.square_distance(i)? > precision::square_confusion() {
            continue;
        }
        let (p1, _p2) = extrema.points(i)?;
        if par_f < p1.parameter() && p1.parameter() < par_l {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The adaptor of the 3d curve of `edge` in place: a copy of the curve, trimmed to the
/// edge range, then moved by the edge location. Standard_NullObject for an edge without
/// 3d curve (a crash natively).
fn placed_curve(edge: &Edge) -> Result<(GeomAdaptorCurve, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    if found.curve.is_null() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "BRepPrimAPI_MakeRevol::CheckValidity: edge without 3d curve; expected a 3d curve",
        ));
    }
    let (first, last) = (found.first, found.last);
    let copy: Handle<dyn GeomCurve> = found.curve.copy()?.cast();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&copy, first, last, true, true)?).cast();
    trimmed.transform(found.location.transformation())?;
    let adaptor = GeomAdaptorCurve::default();
    adaptor.load_range(&trimmed, first, last)?;
    Ok((adaptor, first, last))
}

/// Whether the edge of `curve` lies on the axis: its points at `First + (Last -
/// First) / Ratio`, Ratio = 1..99, all closer to the axis than Precision::Confusion().
fn lies_on_axis(
    curve: &GeomAdaptorCurve,
    first: f64,
    last: f64,
    axe: Axis1,
) -> Result<bool, Failure> {
    let mut ratio = 1;
    loop {
        let point = curve.value(first + (last - first) / f64::from(ratio))?;
        let dist = Line3::from_axis(axe).distance(point);
        ratio += 1;
        if !(dist < precision::confusion() && ratio < 100) {
            break;
        }
    }
    Ok(ratio >= 100)
}

impl BRepPrimAPIMakeRevol {
    /// `CheckValidity(theShape, theA)` (protected natively): no edge that is not
    /// degenerated crosses the axis, except the edges on the axis, which the sweep
    /// accepts; `maker.check_validity(&shape, axis)?`.
    pub fn check_validity(&self, shape: &Shape, a: Axis1) -> Result<bool, Failure> {
        let mut edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            let edge = Edge::unchecked(edges.current().clone());
            edges.next()?;
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let (curve, first, last) = placed_curve(&edge)?;
            if lies_on_axis(&curve, first, last, a)? {
                continue;
            }
            if is_intersect(&curve, a)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
