//! `NewCurve` and `ConvertCurve` of ShapeCustom_BSplineRestriction: the exact conversions
//! of lines, conics and Beziers and the dispatch on trimmed and offset curves.
use super::ShapeCustomBSplineRestriction;
use super::predicates::{is_convert_curve_2d, is_convert_surface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep_tools::NewCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomConic, GeomCurve, GeomLine, GeomOffsetCurve,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeombase::geom_convert::{self, ApproxCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// Whether any curve on surface of the edge satisfies `selects` (the native loops on the
/// BRep_GCurve representations of the TEdge).
pub(super) fn any_curve_on_surface(
    edge: &Edge,
    member: &str,
    selects: impl Fn(&Handle<dyn GeomSurface>, &Handle<dyn Geom2dCurve>) -> bool,
) -> Result<bool, Failure> {
    for representation in edge_tshape(edge, member)?.curves() {
        let gcurve = as_gcurve(&representation);
        let Some(gc) = gcurve.get() else { continue };
        if !gc.is_curve_on_surface() {
            continue;
        }
        if selects(&gc.surface()?, &gc.pcurve()?) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `occ::down_cast<Geom_Curve>(C->Copy())`.
pub(super) fn copied(curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(curve.copy()?.cast())
}

impl ShapeCustomBSplineRestriction {
    /// The `NewCurve` body: the 3D curve converted when it needs it, or copied when a
    /// surface or p-curve of the edge is converted; MSG1 for a B-spline origin, MSG0
    /// otherwise.
    pub(super) fn restricted_curve(
        &self,
        edge: &Edge,
        result: &mut NewCurve,
    ) -> Result<bool, Failure> {
        let state = self.read();
        if !state.approx_curve_3d {
            return Ok(false);
        }
        let found = BRepTool::curve_with_location(edge)?;
        result.location = found.location.clone();
        let mut tol_cur = BRepTool::edge_tolerance(edge)?;
        let parameters = self.parameters()?;
        let limits = self.limits();
        let is_convert =
            any_curve_on_surface(edge, "ShapeCustom_BSplineRestriction::NewCurve", |s, c| {
                state.approx_surface && is_convert_surface(s, limits, &parameters)
                    || state.approx_curve_2d && is_convert_curve_2d(c, limits, &parameters)
            })?;
        let curve = found.curve;
        if curve.is_null() {
            if is_convert {
                (result.curve, result.tolerance) = (curve, tol_cur);
            }
            return Ok(is_convert);
        }
        let range = (found.first, found.last);
        let is_of = !parameters.convert_offset_curv3d();
        let converted = self.convert_curve(
            &curve,
            &mut result.curve,
            is_convert,
            range,
            &mut tol_cur,
            is_of,
        )?;
        result.tolerance = BRepTool::edge_tolerance(edge)?;
        if converted {
            let key: &[u8] = if was_bspline(&curve) {
                b"BSplineRestriction.NewCurve.MSG1"
            } else {
                b"BSplineRestriction.NewCurve.MSG0"
            };
            self.send(&edge.clone().into_shape(), key);
        }
        Ok(converted)
    }

    /// `ConvertCurve(aCurve, C, IsConvert, First, Last, TolCur, IsOf)`: whether `C`
    /// received the conversion of `curve` on `[first, last]` (a copy when `is_convert` and
    /// nothing else applies); `tol_cur` becomes the reached error.
    /// Example: `restriction.convert_curve(&c, &mut out, false, (0., 1.), &mut tol, true)?`.
    pub fn convert_curve(
        &self,
        curve: &Handle<dyn GeomCurve>,
        result: &mut Handle<dyn GeomCurve>,
        is_convert: bool,
        range: (f64, f64),
        tol_cur: &mut f64,
        is_of: bool,
    ) -> Result<bool, Failure> {
        let fallback =
            |result: &mut Handle<dyn GeomCurve>, tol_cur: &mut f64| -> Result<bool, Failure> {
                if is_convert {
                    (*result, *tol_cur) = (copied(curve)?, precision::confusion());
                }
                Ok(is_convert)
            };
        if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            let mut converted = Handle::default();
            if self.convert_curve(
                &trimmed.basis_curve(),
                &mut converted,
                is_convert,
                range,
                tol_cur,
                is_of,
            )? {
                *result = converted;
                return Ok(true);
            }
            return fallback(result, tol_cur);
        }
        if let Some(outcome) = self.convert_exact(curve, result, is_convert, range, tol_cur)? {
            return Ok(outcome);
        }
        let offset = curve.cast::<GeomOffsetCurve>();
        if let Some(offset) = offset.get().filter(|_| is_of) {
            let mut converted = Handle::default();
            if !self.convert_curve(
                &offset.basis_curve(),
                &mut converted,
                is_convert,
                range,
                tol_cur,
                true,
            )? {
                return fallback(result, tol_cur);
            }
            if converted.continuity() != Continuity::C0 {
                *result = Handle::new(GeomOffsetCurve::new(
                    &converted,
                    offset.offset(),
                    offset.direction(),
                    false,
                )?)
                .cast();
                return Ok(true);
            }
            if self.convert_curve(curve, result, is_convert, range, tol_cur, false)? {
                return Ok(true);
            }
            return fallback(result, tol_cur);
        }
        let approximable = !curve.cast::<GeomBSplineCurve>().is_null()
            || !curve.cast::<GeomBezierCurve>().is_null()
            || !offset.is_null() && !is_of;
        if approximable {
            return self.approximate_curve(curve, result, is_convert, range, tol_cur);
        }
        fallback(result, tol_cur)
    }

    // Lines, conics and Beziers with ConvertCurve3d get an exact B-spline (a conic is first
    // approximated with the 2D continuity, a native quirk kept), then a second pass.
    fn convert_exact(
        &self,
        curve: &Handle<dyn GeomCurve>,
        result: &mut Handle<dyn GeomCurve>,
        is_convert: bool,
        (first, last): (f64, f64),
        tol_cur: &mut f64,
    ) -> Result<Option<bool>, Failure> {
        if !self.parameters()?.convert_curve3d() {
            return Ok(None);
        }
        if let Some(line) = curve.cast::<GeomLine>().get() {
            let poles = [line.value(first)?, line.value(last)?];
            let (knots, mults) = ([first, last], [2, 2]);
            let spline = GeomBSplineCurve::new(
                &Array1::from_shared(&poles, 1),
                &Array1::from_shared(&knots, 1),
                &Array1::from_shared(&mults, 1),
                1,
                false,
            )?;
            *result = Handle::new(spline).cast();
            return Ok(Some(true));
        }
        let spline: Handle<dyn GeomCurve> = if !curve.cast::<dyn GeomConic>().is_null() {
            self.conic_spline(curve, first, last)?.cast()
        } else if !curve.cast::<GeomBezierCurve>().is_null() {
            geom_convert::curve_to_bspline_curve(curve, ParameterisationType::QuasiAngular)?.cast()
        } else {
            return Ok(None);
        };
        let mut converted = Handle::default();
        if self.convert_curve(
            &spline,
            &mut converted,
            is_convert,
            (first, last),
            tol_cur,
            false,
        )? {
            *result = converted;
        } else {
            (*result, *tol_cur) = (spline, precision::p_confusion());
        }
        Ok(Some(true))
    }

    fn conic_spline(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let state = self.read();
        // "protection against parabols ets".
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast();
        let approx = ApproxCurve::from_curve(
            &trimmed,
            state.tol_3d,
            state.continuity_2d,
            state.nb_max_seg,
            6,
        )?;
        let spline = if approx.has_result() {
            approx.curve()
        } else {
            geom_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::QuasiAngular)?
        };
        shifted_knots(&spline, first)?;
        Ok(spline)
    }
}

/// Moves the knots so that the curve starts at `first` when it differs by more than
/// Precision::PConfusion.
fn shifted_knots(spline: &Handle<GeomBSplineCurve>, first: f64) -> Result<(), Failure> {
    let Some(s) = spline.get() else { return Ok(()) };
    let shift = first - s.first_parameter();
    if shift.abs() <= precision::p_confusion() {
        return Ok(());
    }
    let knots: Vec<f64> = (1..=s.nb_knots())
        .map(|i| s.knot(i).map(|k| k + shift))
        .collect::<Result<_, _>>()?;
    s.set_knots(&Array1::from_shared(&knots, 1))
}

fn was_bspline(curve: &Handle<dyn GeomCurve>) -> bool {
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        return !trimmed.basis_curve().cast::<GeomBSplineCurve>().is_null();
    }
    !curve.cast::<GeomBSplineCurve>().is_null()
}
