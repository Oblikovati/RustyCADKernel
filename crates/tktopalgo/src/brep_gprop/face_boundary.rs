// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Face.cxx (the
// boundary-curve orders and the knot spans, with the file-static GetRealKnots and
// GetCurveKnots) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundary-curve orders and knots of `BRepGProp_Face`, and the knot spans of the
//! Gauss-Kronrod integration.
use super::face::BRepGPropFace;
use super::face_orders::{
    bspline_curve_knots, bspline_surface_knots, copy_knots, curve_degree_and_knots, l_coeff,
    write_knots,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{epsilon, real_to_int};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomSurfaceOfLinearExtrusion, GeomSweptSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::math;
use std::f64::consts::PI;

/// The order `NL` of `LIntOrder` by boundary-curve type: the degree of a Bezier or
/// B-spline curve, fixed values for the other types (the conics count the spans of
/// their conversion).
fn curve_order(tool: &BRepGPropFace) -> Result<f64, Failure> {
    Ok(match tool.curve.get_type()? {
        CurveType::Line => 1.,
        CurveType::Circle | CurveType::Ellipse | CurveType::Parabola => 2. * 3.,
        CurveType::BezierCurve | CurveType::BSplineCurve => {
            f64::from(curve_degree_and_knots(tool)?.0)
        }
        _ => 3. * 3.,
    })
}

impl BRepGPropFace {
    /// `LIntOrder(Eps)`: the Gauss order along the boundary curve, from the share of the
    /// V range the curve spans, the surface order `SIntOrder(1)` and the curve type;
    /// `tool.l_int_order(1.0e-6)?`.
    pub fn l_int_order(&self, eps: f64) -> Result<i32, Failure> {
        let mut bounds = BoundingBox2::default();
        Add2dCurve::add_adaptor(&self.curve, 1.0e-7, &mut bounds)?;
        let [_, ymin, _, ymax] = bounds.get()?;
        let dv = self.surface.last_v_parameter()? - self.surface.first_v_parameter()?;
        let ratio = if dv > epsilon(1.) {
            std_min((ymax - ymin) / dv, 1.)
        } else {
            1.
        };
        let r_int = real_to_int((f64::from(self.sv_int_subs()?) * ratio).ceil());
        let l_subs = self.l_int_subs()?;
        let spread = (self.s_int_order(1.)? * r_int)
            .checked_div(l_subs)
            .ok_or_else(|| zero_subs(l_subs))?;
        let ns = f64::from(spread.max(1));
        let nl = std_max(curve_order(self)?, ns);
        let nn = if l_subs <= 4 {
            real_to_int((l_coeff(eps) * (nl + 1.)).ceil())
        } else {
            real_to_int(nl + 1.)
        };
        Ok(nn.min(math::gauss_points_max()))
    }

    /// `LIntSubs()`: the interval count of the boundary curve, `NbKnots - 1` for a
    /// B-spline curve; `tool.l_int_subs()?`.
    pub fn l_int_subs(&self) -> Result<i32, Failure> {
        let n = match self.curve.get_type()? {
            CurveType::Circle | CurveType::Ellipse => 4,
            CurveType::BSplineCurve => bspline_curve_knots(self)?.length(),
            _ => 2,
        };
        Ok(n - 1)
    }

    /// `LKnots(Knots)`: the curve range, the thirds of a turn for a circle or ellipse, or
    /// the knots of a B-spline curve; Standard_OutOfRange when `knots` is too short;
    /// `tool.l_knots(&mut knots)?`.
    pub fn l_knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        match self.curve.get_type()? {
            CurveType::Circle | CurveType::Ellipse => write_knots(
                knots,
                &[0.0, PI * 2.0 / 3.0, PI * 4.0 / 3.0, PI * 6.0 / 3.0],
            ),
            CurveType::BSplineCurve => copy_knots(knots, &bspline_curve_knots(self)?),
            _ => write_knots(
                knots,
                &[self.curve.first_parameter()?, self.curve.last_parameter()?],
            ),
        }
    }

    /// `GetUKnots(theUMin, theUMax)`: `theUMin`, the U knots strictly inside the range
    /// (beyond Precision::Confusion()) and `theUMax`, for a B-spline surface or an
    /// extrusion of a B-spline curve when the tool uses spans; `[theUMin, theUMax]`
    /// otherwise; `tool.get_u_knots(0.0, 1.0)?`.
    pub fn get_u_knots(&self, u_min: f64, u_max: f64) -> Result<Array1<'static, f64>, Failure> {
        let kind = self.surface.get_type()?;
        let is_s_bspline = kind == SurfaceType::BSplineSurface;
        // The basis of an extrusion is loaded whatever the span flag, as natively.
        let c_knots = match (is_s_bspline, kind) {
            (false, SurfaceType::SurfaceOfExtrusion) => self.extrusion_bspline_knots()?,
            _ => None,
        };
        if !self.is_use_span {
            return range_knots(u_min, u_max);
        }
        if is_s_bspline {
            return real_knots(u_min, u_max, &bspline_surface_knots(self, true)?);
        }
        match c_knots {
            Some(knots) => real_knots(u_min, u_max, &knots),
            None => range_knots(u_min, u_max),
        }
    }

    /// The knots of the B-spline basis curve of an extrusion, `None` for another basis.
    fn extrusion_bspline_knots(&self) -> Result<Option<Array1<'static, f64>>, Failure> {
        let original = self.surface.transformed_surface().geom_surface_original();
        let extrusion: Handle<GeomSurfaceOfLinearExtrusion> = original.cast();
        let extrusion = extrusion.get().ok_or_else(not_extrusion)?;
        let basis = GeomAdaptorCurve::with_curve(&extrusion.basis_curve())?;
        if basis.get_type()? != CurveType::BSplineCurve {
            return Ok(None);
        }
        let bspline = basis.bspline()?;
        Ok(bspline.get().map(|curve| curve.knots()))
    }

    /// `GetUKnots(theUMin, theUMax, theUKnots)` (deprecated): into `knots`;
    /// `tool.get_u_knots_into(0.0, 1.0, &mut knots)?`.
    pub fn get_u_knots_into(
        &self,
        u_min: f64,
        u_max: f64,
        knots: &mut Array1<'static, f64>,
    ) -> Result<(), Failure> {
        *knots = self.get_u_knots(u_min, u_max)?;
        Ok(())
    }

    /// `GetTKnots(theTMin, theTMax)`: `theTMin`, the knots of a B-spline boundary curve
    /// strictly inside the range and `theTMax` on a B-spline surface when the tool uses
    /// spans (the V knots of the surface are read but not used, as natively);
    /// `[theTMin, theTMax]` otherwise; `tool.get_t_knots(0.0, 1.0)?`.
    pub fn get_t_knots(&self, t_min: f64, t_max: f64) -> Result<Array1<'static, f64>, Failure> {
        let is_bspline = self.surface.get_type()? == SurfaceType::BSplineSurface;
        if !(self.is_use_span && is_bspline) {
            return range_knots(t_min, t_max);
        }
        bspline_surface_knots(self, false)?;
        if self.curve.get_type()? == CurveType::BSplineCurve {
            return real_knots(t_min, t_max, &bspline_curve_knots(self)?);
        }
        range_knots(t_min, t_max)
    }

    /// `GetTKnots(theTMin, theTMax, theTKnots)` (deprecated): into `knots`;
    /// `tool.get_t_knots_into(0.0, 1.0, &mut knots)?`.
    pub fn get_t_knots_into(
        &self,
        t_min: f64,
        t_max: f64,
        knots: &mut Array1<'static, f64>,
    ) -> Result<(), Failure> {
        *knots = self.get_t_knots(t_min, t_max)?;
        Ok(())
    }
}

/// `[theMin, theMax]`, indexed from 1.
fn range_knots(min: f64, max: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut knots = new_knots(2)?;
    knots[1] = min;
    knots[2] = max;
    Ok(knots)
}

fn new_knots(count: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, count).map_err(|error| error.into_failure("BRepGProp_Face knots"))
}

/// `GetRealKnots`: `min`, the knots from the first one above `min + Confusion` to the
/// last one before the first knot above `max - Confusion`, then `max`. An index 0
/// means "not found", as natively (the knot arrays start at 1).
fn real_knots(
    min: f64,
    max: f64,
    knots: &Array1<'_, f64>,
) -> Result<Array1<'static, f64>, Failure> {
    let tol = precision::confusion();
    let (mut start, mut end) = (0, 0);
    let upper = knots.upper();
    for i in knots.lower()..upper {
        if start == 0 && knot(knots, i)? > min + tol {
            start = i;
        }
        if end == 0 && knot(knots, i + 1)? > max - tol {
            end = i;
        }
        if start != 0 && end != 0 {
            break;
        }
    }
    if start == 0 {
        start = upper;
    }
    let count = (end - start + 1).max(0) + 2;
    let mut real = new_knots(count)?;
    real[1] = min;
    real[count] = max;
    for (i, j) in (2..).zip(start..=end) {
        real[i] = knot(knots, j)?;
    }
    Ok(real)
}

fn knot(knots: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    knots
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("BRepGProp_Face GetRealKnots"))
}

fn zero_subs(l_subs: i32) -> Failure {
    Failure::with_type(
        "Standard_DivideByZero",
        &format!("BRepGProp_Face::LIntOrder: LIntSubs() = {l_subs}; expected a positive count"),
        "",
    )
}

fn not_extrusion() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepGProp_Face::GetUKnots: the surface of type SurfaceOfExtrusion is not a \
         Geom_SurfaceOfLinearExtrusion",
        "",
    )
}
