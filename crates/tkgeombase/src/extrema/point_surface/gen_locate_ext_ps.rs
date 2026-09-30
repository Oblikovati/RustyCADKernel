// Rust translation of Extrema_GenLocateExtPS.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenLocateExtPS`: the point-surface extremum nearest to a start `(u, v)`,
//! by the roots of [`FuncPSNorm`] (math_FunctionSetRoot, then Newton) or by the
//! minimum of [`FuncPSDist`] (math_BFGS, then math_FRPR).
use super::func_ps_dist::FuncPSDist;
use super::func_ps_norm::FuncPSNorm;
use crate::extrema::failures::not_done;
use crate::extrema::pon::POnSurf;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{Bfgs, Frpr, FunctionSetRoot, NewtonFunctionSetRoot, Vector};

/// `Extrema_GenLocateExtPS`; `GenLocateExtPS::new(&surface, 1e-9, 1e-9)`.
pub struct GenLocateExtPS<'a> {
    surf: &'a dyn Adaptor3dSurface,
    tol_u: f64,
    tol_v: f64,
    done: bool,
    sq_dist: f64,
    point: POnSurf,
}

// A 1-based math_Vector of two values.
fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (first, second);
    Ok(vector)
}

// The tolerance of `CorrectTol` for one parameter: PConfusion raised by one decade
// per decade of `Epsilon(value)` above `Epsilon(1)`, or None below it.
// The negated test keeps the native NaN semantics.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
// 0.43429 is the native literal, not LOG10_E.
#[allow(clippy::approx_constant)]
fn corrected_tolerance(value: f64) -> Option<f64> {
    let eps_ref: f64 = real::epsilon(1.);
    let eps: f64 = real::epsilon(value);
    if !(eps > eps_ref) {
        return None;
    }
    let tolog10: f64 = 0.43429;
    let n: i32 = real::real_to_int(tolog10 * (eps / eps_ref).ln() + 1.) + 1;
    let mut tol: f64 = precision::p_confusion();
    for _ in 1..=n {
        tol *= 10.;
    }
    Some(tol)
}

// `CorrectTol`: widens the tolerances for large values of UV parameters.
fn correct_tol(u0: f64, v0: f64, tol: &mut Vector<'_>) {
    if let Some(tol_u) = corrected_tolerance(u0) {
        tol[1] = std_max(tol[1], tol_u);
    }
    if let Some(tol_v) = corrected_tolerance(v0) {
        tol[2] = std_max(tol[2], tol_v);
    }
}

// A parameter clamped to the range unless the surface is periodic in it.
fn clamp_unless_periodic(value: f64, periodic: bool, (first, last): (f64, f64)) -> f64 {
    if periodic {
        return value;
    }
    std_min(std_max(value, first), last)
}

impl<'a> GenLocateExtPS<'a> {
    /// `Extrema_GenLocateExtPS(theS, theTolU, theTolV)` (the native defaults are
    /// PConfusion); `GenLocateExtPS::new(&surface, 1e-9, 1e-9)`.
    pub fn new(surface: &'a dyn Adaptor3dSurface, tol_u: f64, tol_v: f64) -> Self {
        Self {
            surf: surface,
            tol_u,
            tol_v,
            done: false,
            sq_dist: -1.0,
            point: POnSurf::default(),
        }
    }
    /// `IsMinDist(theP, theS, theU0, theV0)`: no neighbour of `(U0, V0)` one
    /// resolution away (clamped to a non-periodic range) is nearer to P;
    /// `GenLocateExtPS::is_min_dist(point, &surface, 0.5, 0.5)?`.
    pub fn is_min_dist(
        point: Point3,
        surface: &dyn Adaptor3dSurface,
        u0: f64,
        v0: f64,
    ) -> Result<bool, Failure> {
        let du: f64 = std_max(
            surface.u_resolution(10. * precision::confusion())?,
            10. * precision::p_confusion(),
        );
        let dv: f64 = std_max(
            surface.v_resolution(10. * precision::confusion())?,
            10. * precision::p_confusion(),
        );
        let d0: f64 = point.square_distance(&surface.value(u0, v0)?);
        for iu in -1..=1 {
            let u_range = (surface.first_u_parameter()?, surface.last_u_parameter()?);
            let u: f64 =
                clamp_unless_periodic(u0 + f64::from(iu) * du, surface.is_u_periodic()?, u_range);
            for iv in -1..=1 {
                if iu == 0 && iv == 0 {
                    continue;
                }
                let v_range = (surface.first_v_parameter()?, surface.last_v_parameter()?);
                let v: f64 = clamp_unless_periodic(
                    v0 + f64::from(iv) * dv,
                    surface.is_v_periodic()?,
                    v_range,
                );
                if point.square_distance(&surface.value(u, v)?) < d0 {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    /// `Perform(theP, theU0, theV0, isDistanceCriteria)` (the native default is
    /// false, the normal projection); `extrema.perform(point, 0.5, 0.5, false)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        u0: f64,
        v0: f64,
        is_distance_criteria: bool,
    ) -> Result<(), Failure> {
        self.done = false;
        if is_distance_criteria {
            return self.perform_distance(point, pair(u0, v0)?);
        }
        self.perform_normal(point, u0, v0)
    }

    // The distance criteria: math_BFGS, then math_FRPR when it fails.
    fn perform_distance(&mut self, point: Point3, start: Vector<'static>) -> Result<(), Failure> {
        let rel_tol: f64 = 1.0e-8;
        let mut function = FuncPSDist::new(self.surf, point);
        let mut solver = Bfgs::with_options(2, rel_tol, 200, 1.0e-12)?;
        solver.perform(&mut function, &start)?;
        function.take_failure()?;
        let location: Vector<'static>;
        if !solver.is_done() {
            let mut solver1 = Frpr::new(&function, rel_tol)?;
            solver1.perform(&mut function, &start)?;
            function.take_failure()?;
            if !solver1.is_done() {
                return Ok(());
            }
            location = solver1.location()?.clone();
            self.sq_dist = solver1.minimum()?;
        } else {
            location = solver.location()?.clone();
            self.sq_dist = solver.minimum()?;
        }
        let (u, v) = (location[1], location[2]);
        self.point.set_parameters(u, v, self.surf.value(u, v)?);
        self.done = true;
        Ok(())
    }

    // The normal projection criteria: math_FunctionSetRoot, then
    // math_NewtonFunctionSetRoot when it fails or when the tolerances were corrected.
    fn perform_normal(&mut self, point: Point3, u0: f64, v0: f64) -> Result<(), Failure> {
        let start: Vector<'static> = pair(u0, v0)?;
        let bound_inf = pair(
            self.surf.first_u_parameter()?,
            self.surf.first_v_parameter()?,
        )?;
        let bound_sup = pair(self.surf.last_u_parameter()?, self.surf.last_v_parameter()?)?;
        let mut function = FuncPSNorm::with_point_surface(point, self.surf);
        let mut tol: Vector<'static> = pair(self.tol_u, self.tol_v)?;
        if self.surf.get_type()? == SurfaceType::BSplineSurface {
            correct_tol(u0, v0, &mut tol);
        }
        let is_correct_tol: bool = (tol[1] - self.tol_u).abs() > precision::p_confusion()
            || (tol[2] - self.tol_v).abs() > precision::p_confusion();
        let mut sr = FunctionSetRoot::new(&function, &tol, 100)?;
        sr.perform_bounded(&mut function, &start, &bound_inf, &bound_sup, false)?;
        function.take_failure()?;
        if !sr.is_done() || is_correct_tol {
            if is_correct_tol {
                tol = pair(self.tol_u, self.tol_v)?;
            }
            let mut nsr = NewtonFunctionSetRoot::new(&function, &tol, precision::confusion(), 100)?;
            nsr.perform_bounded(&mut function, &start, &bound_inf, &bound_sup)?;
            function.take_failure()?;
            if !sr.is_done() && !nsr.is_done() {
                return Ok(());
            }
        }
        self.keep_nearest(&function)
    }

    // The nearest root the function saved (the first of equal distances).
    fn keep_nearest(&mut self, function: &FuncPSNorm<'_>) -> Result<(), Failure> {
        self.sq_dist = function.square_distance(1)?;
        self.point = function.point(1)?;
        for i in 2..=function.nb_ext() {
            if function.square_distance(i)? < self.sq_dist {
                self.sq_dist = function.square_distance(i)?;
                self.point = function.point(i)?;
            }
        }
        self.done = true;
        Ok(())
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `SquareDistance()`: StdFail_NotDone when not done; `extrema.square_distance()?`.
    pub fn square_distance(&self) -> Result<f64, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenLocateExtPS::SquareDistance()"));
        }
        Ok(self.sq_dist)
    }
    /// `Point()`: StdFail_NotDone when not done; `extrema.point()?`.
    pub fn point(&self) -> Result<POnSurf, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenLocateExtPS::Point()"));
        }
        Ok(self.point)
    }
}
