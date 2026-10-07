//! `NewCurve2d` and `ConvertCurve2d` of ShapeCustom_BSplineRestriction.
use super::ShapeCustomBSplineRestriction;
use super::curve3d::any_curve_on_surface;
use super::predicates::{is_convert_curve_2d, is_convert_curve_3d, is_convert_surface};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dConic, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve,
    Geom2dTrimmedCurve,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::geom2d_convert::{self, ApproxCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The inputs of `NewCurve2d`.
pub(super) struct PCurveRequest<'a> {
    pub edge: &'a Edge,
    pub face: &'a Face,
    pub new_edge: &'a Edge,
}

impl ShapeCustomBSplineRestriction {
    /// The `NewCurve2d` body: the p-curve converted (or copied when the surface, the 3D
    /// curve or another p-curve of the edge is converted); without the 2D flag only the
    /// copy; a new edge gets the p-curve range when nothing changed.
    pub(super) fn restricted_pcurve(
        &self,
        request: PCurveRequest<'_>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let state = self.read();
        if !state.approx_curve_2d && !state.approx_surface {
            return Ok(false);
        }
        let (edge, face) = (request.edge, request.face);
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
        let edge_tol = BRepTool::edge_tolerance(edge)?;
        let mut tol_cur = adaptor
            .u_resolution(edge_tol)?
            .min(adaptor.v_resolution(edge_tol)?);
        let found = BRepTool::curve_on_face(edge, face)?;
        if found.pcurve.is_null() {
            return Ok(false);
        }
        let is_convert = self.edge_needs_conversion(edge, &surface)?;
        if !state.approx_curve_2d {
            if is_convert {
                *curve = found.pcurve.copy()?.cast();
            }
            return Ok(is_convert);
        }
        let is_of = !self.parameters()?.convert_offset_curv2d();
        let range = (found.first, found.last);
        let converted =
            self.convert_curve_2d(&found.pcurve, curve, is_convert, range, &mut tol_cur, is_of)?;
        *tolerance = BRepTool::edge_tolerance(edge)?;
        if !converted && !request.new_edge.is_same(edge) {
            BRepBuilder::default().range(request.new_edge, range.0, range.1, false)?;
        }
        Ok(converted)
    }

    fn edge_needs_conversion(
        &self,
        edge: &Edge,
        surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    ) -> Result<bool, Failure> {
        let state = self.read();
        let parameters = self.parameters()?;
        let limits = self.limits();
        let curve = BRepTool::curve_with_location(edge)?.curve;
        if state.approx_surface && is_convert_surface(surface, limits, &parameters)
            || state.approx_curve_3d && is_convert_curve_3d(&curve, limits, &parameters)
        {
            return Ok(true);
        }
        any_curve_on_surface(
            edge,
            "ShapeCustom_BSplineRestriction::NewCurve2d",
            |s, c| {
                state.approx_surface && is_convert_surface(s, limits, &parameters)
                    || state.approx_curve_2d && is_convert_curve_2d(c, limits, &parameters)
            },
        )
    }

    /// `ConvertCurve2d(aCurve, C, IsConvert, First, Last, TolCur, IsOf)`: the 2D twin of
    /// `ConvertCurve` (copies get Precision::PConfusion). Native quirk kept: when an offset
    /// basis converts to C0 and the second pass succeeds, the result is reported but `C` is
    /// left unchanged.
    /// Example: `restriction.convert_curve_2d(&c, &mut out, false, (0., 1.), &mut tol, true)?`.
    pub fn convert_curve_2d(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        result: &mut Handle<dyn Geom2dCurve>,
        is_convert: bool,
        range: (f64, f64),
        tol_cur: &mut f64,
        is_of: bool,
    ) -> Result<bool, Failure> {
        let fallback =
            |result: &mut Handle<dyn Geom2dCurve>, tol_cur: &mut f64| -> Result<bool, Failure> {
                if is_convert {
                    (*result, *tol_cur) = (curve.copy()?.cast(), precision::p_confusion());
                }
                Ok(is_convert)
            };
        if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
            let mut converted = Handle::default();
            if self.convert_curve_2d(
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
        if let Some(outcome) = self.convert_exact_2d(curve, result, is_convert, range, tol_cur)? {
            return Ok(outcome);
        }
        let offset = curve.cast::<Geom2dOffsetCurve>();
        if let Some(offset) = offset.get().filter(|_| is_of) {
            let mut converted = Handle::default();
            if !self.convert_curve_2d(
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
                *result =
                    Handle::new(Geom2dOffsetCurve::new(&converted, offset.offset(), false)?).cast();
                return Ok(true);
            }
            if self.convert_curve_2d(curve, &mut converted, is_convert, range, tol_cur, false)? {
                return Ok(true);
            }
            return fallback(result, tol_cur);
        }
        let approximable = !curve.cast::<Geom2dBSplineCurve>().is_null()
            || !curve.cast::<Geom2dBezierCurve>().is_null()
            || !offset.is_null() && !is_of;
        if approximable {
            return self.approximate_curve_2d(curve, result, is_convert, range, tol_cur);
        }
        fallback(result, tol_cur)
    }

    fn convert_exact_2d(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        result: &mut Handle<dyn Geom2dCurve>,
        is_convert: bool,
        (first, last): (f64, f64),
        tol_cur: &mut f64,
    ) -> Result<Option<bool>, Failure> {
        if !self.parameters()?.convert_curve2d() {
            return Ok(None);
        }
        if let Some(line) = curve.cast::<Geom2dLine>().get() {
            let poles = [line.value(first)?, line.value(last)?];
            let (knots, mults) = ([first, last], [2, 2]);
            let spline = Geom2dBSplineCurve::new(
                &Array1::from_shared(&poles, 1),
                &Array1::from_shared(&knots, 1),
                &Array1::from_shared(&mults, 1),
                1,
                false,
            )?;
            *result = Handle::new(spline).cast();
            return Ok(Some(true));
        }
        let spline: Handle<dyn Geom2dCurve> = if !curve.cast::<dyn Geom2dConic>().is_null() {
            self.conic_spline_2d(curve, first, last)?.cast()
        } else if !curve.cast::<Geom2dBezierCurve>().is_null() {
            geom2d_convert::curve_to_bspline_curve(curve, ParameterisationType::QuasiAngular)?
                .cast()
        } else {
            return Ok(None);
        };
        let mut converted = Handle::default();
        if self.convert_curve_2d(
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

    fn conic_spline_2d(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let state = self.read();
        let trimmed: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
        let approx = ApproxCurve::from_curve(
            &trimmed,
            state.tol_2d,
            state.continuity_2d,
            state.nb_max_seg,
            6,
        )?;
        let spline = if approx.has_result() {
            approx.curve()
        } else {
            geom2d_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::QuasiAngular)?
        };
        if let Some(s) = spline.get() {
            let shift = first - s.first_parameter();
            if shift.abs() > precision::p_confusion() {
                let knots: Vec<f64> = (1..=s.nb_knots())
                    .map(|i| s.knot(i).map(|k| k + shift))
                    .collect::<Result<_, _>>()?;
                s.set_knots(&Array1::from_shared(&knots, 1))?;
            }
        }
        Ok(spline)
    }
}
