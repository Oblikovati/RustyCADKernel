// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Face.cxx (the
// integration orders, subdivisions and knots) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The integration orders, interval counts and knots of `BRepGProp_Face`.
use super::face::BRepGPropFace;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::math;
use std::f64::consts::PI;

/// The knots of an analytic surface or curve: thirds of the full turn.
const TURN_THIRDS: [f64; 4] = [0.0, PI * 2.0 / 3.0, PI * 4.0 / 3.0, PI * 6.0 / 3.0];

/// The `(Degree, NbKnots)` of the Bezier or B-spline geometry a face tool reads.
fn degree_and_knots_u(tool: &BRepGPropFace) -> Result<(i32, i32), Failure> {
    let original = tool.surface.transformed_surface().geom_surface_original();
    if let Some(bezier) = original.cast::<GeomBezierSurface>().get() {
        return Ok((bezier.u_degree(), 2));
    }
    let bspline: Handle<GeomBSplineSurface> = original.cast();
    let bspline = bspline
        .get()
        .ok_or_else(|| not_spline("GeomSurfaceOriginal"))?;
    Ok((bspline.u_degree(), bspline.nb_u_knots()))
}

fn degree_and_knots_v(tool: &BRepGPropFace) -> Result<(i32, i32), Failure> {
    let original = tool.surface.transformed_surface().geom_surface_original();
    if let Some(bezier) = original.cast::<GeomBezierSurface>().get() {
        return Ok((bezier.v_degree(), 2));
    }
    let bspline: Handle<GeomBSplineSurface> = original.cast();
    let bspline = bspline
        .get()
        .ok_or_else(|| not_spline("GeomSurfaceOriginal"))?;
    Ok((bspline.v_degree(), bspline.nb_v_knots()))
}

/// The U knots (or V knots) of a face whose adaptor type is a B-spline surface.
pub(super) fn bspline_surface_knots(
    tool: &BRepGPropFace,
    in_u: bool,
) -> Result<Array1<'static, f64>, Failure> {
    let original = tool.surface.transformed_surface().geom_surface_original();
    let bspline: Handle<GeomBSplineSurface> = original.cast();
    let bspline = bspline
        .get()
        .ok_or_else(|| not_spline("GeomSurfaceOriginal"))?;
    Ok(if in_u {
        bspline.u_knots()
    } else {
        bspline.v_knots()
    })
}

/// The degree and knot count of the boundary curve (Bezier: 2 knots).
pub(super) fn curve_degree_and_knots(tool: &BRepGPropFace) -> Result<(i32, i32), Failure> {
    let curve = tool.curve.curve();
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        return Ok((bezier.degree(), 2));
    }
    let bspline: Handle<Geom2dBSplineCurve> = curve.cast();
    let bspline = bspline.get().ok_or_else(|| not_spline("Curve"))?;
    Ok((bspline.degree(), bspline.nb_knots()))
}

/// The knots of the boundary curve of a tool whose curve type is a B-spline curve.
pub(super) fn bspline_curve_knots(tool: &BRepGPropFace) -> Result<Array1<'static, f64>, Failure> {
    let bspline: Handle<Geom2dBSplineCurve> = tool.curve.curve().cast();
    let bspline = bspline.get().ok_or_else(|| not_spline("Curve"))?;
    Ok(bspline.knots())
}

fn not_spline(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepGProp_Face: {what} is not the Bezier or B-spline geometry its type says"),
        "",
    )
}

/// `SCoeff(Eps)` (APO 17.04.2002, OCC104: coefficients fitted on hard examples).
fn s_coeff(eps: f64) -> f64 {
    if eps < 0.1 {
        -0.15 * (1.0 + eps.log10()) + 0.75
    } else {
        0.75
    }
}

/// `LCoeff(Eps)` (OCC104).
pub(super) fn l_coeff(eps: f64) -> f64 {
    if eps < 0.1 {
        -0.50 * (1.0 + eps.log10()) + 0.25
    } else {
        0.25
    }
}

impl BRepGPropFace {
    /// `UIntegrationOrder()`: `max(8, 2 Nu)` with Nu 4 for a plane, `max(4, deg + 1)`
    /// for a Bezier, `max(4, (deg + 1)(nbknots - 1))` for a B-spline surface, 9 else;
    /// `tool.u_integration_order()?`.
    pub fn u_integration_order(&self) -> Result<i32, Failure> {
        let nu = match self.surface.get_type()? {
            SurfaceType::Plane => 4,
            SurfaceType::BezierSurface => 4.max(degree_and_knots_u(self)?.0 + 1),
            SurfaceType::BSplineSurface => {
                let (degree, knots) = degree_and_knots_u(self)?;
                4.max((degree + 1) * (knots - 1))
            }
            _ => 9,
        };
        Ok(8.max(2 * nu))
    }

    /// `VIntegrationOrder()`: [`BRepGPropFace::u_integration_order`] in V;
    /// `tool.v_integration_order()?`.
    pub fn v_integration_order(&self) -> Result<i32, Failure> {
        let nv = match self.surface.get_type()? {
            SurfaceType::Plane => 4,
            SurfaceType::BezierSurface => 4.max(degree_and_knots_v(self)?.0 + 1),
            SurfaceType::BSplineSurface => {
                let (degree, knots) = degree_and_knots_v(self)?;
                4.max((degree + 1) * (knots - 1))
            }
            _ => 9,
        };
        Ok(8.max(2 * nv))
    }

    /// `IntegrationOrder()`: `max(4, 2 N)` with N 2 for a line, 9 for a conic,
    /// `deg + 1` for a Bezier, `(deg + 1)(nbknots - 1)` for a B-spline boundary curve;
    /// `tool.integration_order()?`.
    pub fn integration_order(&self) -> Result<i32, Failure> {
        let n = match self.curve.get_type()? {
            CurveType::Line => 2,
            CurveType::BezierCurve => curve_degree_and_knots(self)?.0 + 1,
            CurveType::BSplineCurve => {
                let (degree, knots) = curve_degree_and_knots(self)?;
                (degree + 1) * (knots - 1)
            }
            _ => 9,
        };
        Ok(4.max(2 * n))
    }

    /// `SIntOrder(Eps)`: the Gauss order of the adaptive surface integration,
    /// `min(ceil(SCoeff(Eps) max(Nu + 1, Nv + 1)), GaussPointsMax)`;
    /// `tool.s_int_order(1.0e-6)?`.
    pub fn s_int_order(&self, eps: f64) -> Result<i32, Failure> {
        let (nu, nv) = match self.surface.get_type()? {
            SurfaceType::Plane => (1, 1),
            SurfaceType::Cylinder | SurfaceType::Cone => (2, 1),
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
                (degree_and_knots_u(self)?.0, degree_and_knots_v(self)?.0)
            }
            _ => (2, 2),
        };
        let order = real_to_int((s_coeff(eps) * f64::from((nu + 1).max(nv + 1))).ceil());
        Ok(order.min(math::gauss_points_max()))
    }

    /// `SUIntSubs()`: the U interval count, `NbUKnots - 1` for a B-spline surface;
    /// `tool.su_int_subs()?`.
    pub fn su_int_subs(&self) -> Result<i32, Failure> {
        let n = match self.surface.get_type()? {
            SurfaceType::Plane | SurfaceType::BezierSurface => 2,
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus => 4,
            SurfaceType::BSplineSurface => degree_and_knots_u(self)?.1,
            _ => 2,
        };
        Ok(n - 1)
    }

    /// `SVIntSubs()`: the V interval count, `NbVKnots - 1` for a B-spline surface;
    /// `tool.sv_int_subs()?`.
    pub fn sv_int_subs(&self) -> Result<i32, Failure> {
        let n = match self.surface.get_type()? {
            SurfaceType::Sphere => 3,
            SurfaceType::Torus => 4,
            SurfaceType::BSplineSurface => degree_and_knots_v(self)?.1,
            _ => 2,
        };
        Ok(n - 1)
    }
}

/// Writes `values` from index 1 of `knots`; Standard_OutOfRange past its end.
pub(super) fn write_knots(knots: &mut Array1<'_, f64>, values: &[f64]) -> Result<(), Failure> {
    for (index, value) in (1..).zip(values) {
        knots
            .set_value(index, *value)
            .map_err(|error| error.into_failure("BRepGProp_Face knots"))?;
    }
    Ok(())
}

/// Copies `source(i)` into `knots(i)` for every index of `knots`; Standard_OutOfRange
/// for an index outside `source`.
pub(super) fn copy_knots(
    knots: &mut Array1<'_, f64>,
    source: &Array1<'_, f64>,
) -> Result<(), Failure> {
    for index in knots.lower()..=knots.upper() {
        let value = *source
            .value(index)
            .map_err(|error| error.into_failure("BRepGProp_Face knots"))?;
        knots
            .set_value(index, value)
            .map_err(|error| error.into_failure("BRepGProp_Face knots"))?;
    }
    Ok(())
}

impl BRepGPropFace {
    /// `UKnots(Knots)`: the U knots of the adaptive surface integration (the U bounds
    /// of a plane or other surface, the thirds of a turn for a quadric, the knots of a
    /// B-spline surface); Standard_OutOfRange when `knots` is too short;
    /// `tool.u_knots(&mut knots)?`.
    pub fn u_knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        match self.surface.get_type()? {
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus => write_knots(knots, &TURN_THIRDS),
            SurfaceType::BSplineSurface => copy_knots(knots, &bspline_surface_knots(self, true)?),
            _ => write_knots(knots, &self.u_range()?),
        }
    }

    /// `VKnots(Knots)`: the V knots: the *U* bounds for a plane, cylinder, cone and other
    /// surface (as natively), the half turns of a sphere, the thirds of a turn for a
    /// torus, the V knots of a B-spline surface; Standard_OutOfRange when `knots` is too
    /// short; `tool.v_knots(&mut knots)?`.
    pub fn v_knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        match self.surface.get_type()? {
            SurfaceType::Sphere => write_knots(knots, &[-PI / 2.0, 0.0, PI / 2.0]),
            SurfaceType::Torus => write_knots(knots, &TURN_THIRDS),
            SurfaceType::BSplineSurface => copy_knots(knots, &bspline_surface_knots(self, false)?),
            _ => write_knots(knots, &self.u_range()?),
        }
    }

    fn u_range(&self) -> Result<[f64; 2], Failure> {
        Ok([
            self.surface.first_u_parameter()?,
            self.surface.last_u_parameter()?,
        ])
    }
}
