// Rust translation of BlendFunc_GenChamfer.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members shared by the chamfer functions (`BlendFunc_GenChamfer` and the members
//! `BlendFunc_ChAsym` repeats): linear sections of two poles, the bounds and tolerances of
//! the two surfaces, and the sections at a point of a blend line.
use super::linalg::{vector, vget, widen};
use super::package::{live_curve, live_surface, next_shape};
use super::two_surfaces::point_parameters;
use crate::blend::{BlendFunction, BlendPoint, SectionPoles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::elclib::line_parameter;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Line3, Point2, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_GenChamfer`: a chamfer function between two surfaces along a guide, whose
/// section is set by two distances (or a distance and an angle) and a choice.
///
/// ```ignore
/// let mut chamfer: Box<dyn GenChamferFunction> = Box::new(Chamfer::new(&s1, &s2, &guide));
/// chamfer.set_distances(1.0, 2.0, 1);
/// ```
pub trait GenChamferFunction: BlendFunction {
    /// `Set(Dist1, Dist2, Choix)`: the distances (or throat) and the choice.
    fn set_distances(&mut self, dist1: f64, dist2: f64, choix: i32);

    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`: the line from the point of the
    /// first surface to the point of the second one and the parameter range between them;
    /// `let (pdeb, pfin, line) = chamfer.section_line([u1, v1, u2, v2])?;`.
    fn section_line(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure>;
}

/// The equations of a chamfer function, as the shared members call them.
pub(crate) trait ChamferEquations: BlendFunction {
    fn chamfer_core(&self) -> &GenChamferCore;
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure>;
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure>;
    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure>;
}

/// The fields of `BlendFunc_GenChamfer`.
pub(crate) struct GenChamferCore {
    pub(crate) surf1: Handle<dyn Adaptor3dSurface>,
    pub(crate) surf2: Handle<dyn Adaptor3dSurface>,
    pub(crate) curv: Handle<dyn Adaptor3dCurve>,
    pub(crate) choix: i32,
    pub(crate) tol: f64,
    pub(crate) distmin: f64,
}

impl GenChamferCore {
    /// `BlendFunc_GenChamfer(S1, S2, CG)`: choice 0, tolerance 0, no distance yet.
    pub(crate) fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        cg: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            curv: cg.clone(),
            choix: 0,
            tol: 0.0,
            distmin: f64::MAX,
        }
    }

    /// `GetTolerance(Tolerance, Tol)`: the resolutions of both surfaces.
    pub(crate) fn get_tolerance(
        &self,
        tolerance: &mut Vector<'_>,
        tol: f64,
    ) -> Result<(), Failure> {
        let (s1, s2) = (live_surface(&self.surf1)?, live_surface(&self.surf2)?);
        let values = [
            s1.u_resolution(tol)?,
            s1.v_resolution(tol)?,
            s2.u_resolution(tol)?,
            s2.v_resolution(tol)?,
        ];
        for (i, value) in (1..).zip(values) {
            super::linalg::vset(tolerance, i, value)?;
        }
        Ok(())
    }

    /// `GetBounds(InfBound, SupBound)`: the bounds of both surfaces widened by their range.
    pub(crate) fn get_bounds(
        &self,
        inf: &mut Vector<'_>,
        sup: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (s1, s2) = (live_surface(&self.surf1)?, live_surface(&self.surf2)?);
        let infs = [
            s1.first_u_parameter()?,
            s1.first_v_parameter()?,
            s2.first_u_parameter()?,
            s2.first_v_parameter()?,
        ];
        let sups = [
            s1.last_u_parameter()?,
            s1.last_v_parameter()?,
            s2.last_u_parameter()?,
            s2.last_v_parameter()?,
        ];
        for (i, (a, b)) in (1..).zip(infs.into_iter().zip(sups)) {
            super::linalg::vset(inf, i, a)?;
            super::linalg::vset(sup, i, b)?;
        }
        (1..=4).try_for_each(|i| widen(inf, sup, i))
    }

    /// The line of `GenChamferFunction::section_line`.
    pub(crate) fn section_line(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
        let pts1 = live_surface(&self.surf1)?.value(uv[0], uv[1])?;
        let pts2 = live_surface(&self.surf2)?.value(uv[2], uv[3])?;
        let dir = Direction3::from_vector(Vector3::between(pts1, pts2))?;
        let line = Line3::new(pts1, dir);
        Ok((0., line_parameter(line.position(), pts2), line))
    }

    /// `NbIntervals(S)`: the intervals of the guide at the next continuity.
    pub(crate) fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        live_curve(&self.curv)?.nb_intervals(next_shape(shape))
    }

    /// `Intervals(T, S)`.
    pub(crate) fn intervals(
        &self,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        live_curve(&self.curv)?.intervals(bounds, next_shape(shape))
    }

    /// `Resolution(IC2d, Tol, TolU, TolV)`: the resolutions of surface 1 for `IC2d == 1`,
    /// of surface 2 otherwise.
    pub(crate) fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let surface = live_surface(if index == 1 { &self.surf1 } else { &self.surf2 })?;
        Ok((surface.u_resolution(tol)?, surface.v_resolution(tol)?))
    }
}

/// `GetShape`: two poles, two knots, degree 1.
pub(crate) fn linear_shape() -> (i32, i32, i32, i32) {
    (2, 2, 1, 2)
}

/// `Knots(TKnots)`: 0 and 1.
pub(crate) fn linear_knots(knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
    put(knots, 1, 0.)?;
    put(knots, 2, 1.)
}

/// `Mults(TMults)`: 2 and 2.
pub(crate) fn linear_mults(mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
    put(mults, 1, 2)?;
    put(mults, 2, 2)
}

/// `Section(P, Poles, Poles2d, Weigths)`: `Set(Param)`, `Value` at the point, then the two
/// points with unit weights.
pub(crate) fn section_d0<F: ChamferEquations>(
    function: &mut F,
    point: &BlendPoint,
    values: SectionPoles<'_, '_, '_, '_>,
) -> Result<(), Failure> {
    let x = point_parameters(point)?;
    let uv = uv_of(&x)?;
    put_2d(values.poles2d, uv)?;
    function.set_param(point.parameter())?;
    let mut f = vector(4)?;
    function.try_value(&x, &mut f)?;
    let (low, upp) = (values.poles.lower(), values.poles.upper());
    put(values.poles, low, function.point_on_s1())?;
    put(values.poles, upp, function.point_on_s2())?;
    put(values.weights, low, 1.0)?;
    put(values.weights, upp, 1.0)
}

/// The four parameters `(u1, v1, u2, v2)` of `x`.
pub(crate) fn uv_of(x: &Vector<'_>) -> Result<[f64; 4], Failure> {
    Ok([vget(x, 1)?, vget(x, 2)?, vget(x, 3)?, vget(x, 4)?])
}

/// `Poles2d(Lower) = (u1, v1); Poles2d(Upper) = (u2, v2)`.
fn put_2d(poles2d: &mut Array1<'_, Point2>, uv: [f64; 4]) -> Result<(), Failure> {
    let (low, upp) = (poles2d.lower(), poles2d.upper());
    put(poles2d, low, Point2::new(uv[0], uv[1]))?;
    put(poles2d, upp, Point2::new(uv[2], uv[3]))
}

/// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)` of
/// `BlendFunc_GenChamfer`: `Set(Param)`, `Values`, `IsSolution` with the tolerance of the
/// last `IsSolution`, then the points and, off a tangency point, their tangents.
pub(crate) fn gen_section_d1<F: ChamferEquations>(
    function: &mut F,
    point: &BlendPoint,
    values: SweepD1<'_, '_>,
) -> Result<bool, Failure> {
    let sol = point_parameters(point)?;
    function.set_param(point.parameter())?;
    let (mut valsol, mut gradsol) = (vector(4)?, super::linalg::matrix(4, 4)?);
    function.try_values(&sol, &mut valsol, &mut gradsol)?;
    let tol = function.chamfer_core().tol;
    function.is_solution(&sol, tol)?;
    let istgt = function.is_tangency_point();
    put_2d(values.poles2d, uv_of(&sol)?)?;
    if !istgt {
        put_tangents_2d(
            function,
            values.d_poles2d,
            values.poles2d.lower(),
            values.poles2d.upper(),
        )?;
    }
    let (low, upp) = (values.poles.lower(), values.poles.upper());
    put(values.poles, low, function.point_on_s1())?;
    put(values.poles, upp, function.point_on_s2())?;
    put(values.weights, low, 1.0)?;
    put(values.weights, upp, 1.0)?;
    if !istgt {
        put_tangents_3d(function, values.d_poles, values.d_weights, (low, upp))?;
    }
    Ok(!istgt)
}

/// `DPoles2d(Poles2d.Lower/Upper)` from `Tangent2dOnS1/S2`.
pub(crate) fn put_tangents_2d<F: BlendFunction + ?Sized>(
    function: &F,
    d_poles2d: &mut Array1<'_, Vector2>,
    low: i32,
    upp: i32,
) -> Result<(), Failure> {
    put(d_poles2d, low, function.tangent2d_on_s1()?)?;
    put(d_poles2d, upp, function.tangent2d_on_s2()?)
}

/// `DPoles` from `TangentOnS1/S2` and null weight derivatives.
pub(crate) fn put_tangents_3d<F: BlendFunction + ?Sized>(
    function: &F,
    d_poles: &mut Array1<'_, Vector3>,
    d_weights: &mut Array1<'_, f64>,
    (low, upp): (i32, i32),
) -> Result<(), Failure> {
    put(d_poles, low, function.tangent_on_s1()?)?;
    put(d_poles, upp, function.tangent_on_s2()?)?;
    put(d_weights, low, 0.0)?;
    put(d_weights, upp, 0.0)
}

/// `Tangent(U1, V1, U2, V2, TgF, TgL, NmF, NmL)` of the chamfer functions: the normals of
/// both surfaces and the tangents `nplan ^ N`, reversed by the choice.
pub(crate) fn chamfer_tangent(
    core: &GenChamferCore,
    nplan: Vector3,
    uv: [f64; 4],
) -> Result<crate::blend::SectionEndFrames, Failure> {
    let d1 = live_surface(&core.surf1)?.d1(uv[0], uv[1])?;
    let norm_first = d1.d1u.crossed(d1.d1v);
    let d2 = live_surface(&core.surf2)?.d1(uv[2], uv[3])?;
    let norm_last = d2.d1u.crossed(d2.d1v);
    let mut tg_first = nplan.crossed(norm_first).normalized()?;
    let mut tg_last = nplan.crossed(norm_last).normalized()?;
    let choix = core.choix;
    let rev_f = matches!(choix, 2 | 5 | 3 | 8);
    let rev_l = matches!(choix, 2 | 5 | 4 | 7);
    if rev_f {
        tg_first.reverse();
    }
    if rev_l {
        tg_last.reverse();
    }
    Ok(crate::blend::SectionEndFrames {
        tg_first,
        tg_last,
        norm_first,
        norm_last,
    })
}

/// The `GetSectionSize` failure of the non-rational chamfers ("not implemented, not
/// necessary as not rational").
pub(crate) fn section_size_not_implemented(class: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("{class}::GetSectionSize()"),
    )
}

/// `Values(X, F, D)` of `BlendFunc_GenChamfer`: `Value`, then `Derivatives` only when
/// `Value` succeeded.
pub(crate) fn value_then_derivatives<F: ChamferEquations + ?Sized>(
    function: &mut F,
    x: &Vector<'_>,
    f: &mut Vector<'_>,
    d: &mut Matrix<'_>,
) -> Result<bool, Failure> {
    let val = function.try_value(x, f)?;
    Ok(val && function.try_derivatives(x, d)?)
}
