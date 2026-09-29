// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_BSpGradient.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_BSpGradient` (AppParCurves_BSpGradient.gxx): the parameters of a
//! B-spline fit improved by one projection step then BFGS.
use super::Constraints;
use super::arrays::{line_points, scratch};
use super::bsp_function::BSpFunction;
use super::constraint::Constraint;
use super::function::point_constraint;
use super::gradient::projected;
use super::gradient_bfgs::GradientBfgs;
use super::least_square::LeastSquare;
use super::line_tool::LineTool;
use super::multi_bsp_curve::MultiBSpCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

/// The BFGS tolerance of the native gradients.
const BFGS_EPS: f64 = 1.0e-07;

/// `AppParCurves_BSpGradient`: generic over the MultiLine (the native NbIterations
/// default is 1). With tangency extremities the lambdas come from a first
/// least-squares fit unless given.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::app_par_curves::{BSpGradient, BSpGradientInput, MultiCurveEval};
/// # use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple};
/// # use rustycad_tkernel::ncollection::{Array1, HArray1};
/// # use rustycad_tkernel::standard::Handle;
/// # use rustycad_tkmath::gp::Point3;
/// # use rustycad_tkmath::math::Vector;
/// // Six points of the parabola y = x^2, AppDef_MultiLine read by AppDef_MyLineTool.
/// let points: Vec<Point3> = (0..6).map(|i| {
///     let t = f64::from(i) / 5.0;
///     Point3::new(t, t * t, 0.0)
/// }).collect();
/// let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
/// # let ends = [ConstraintCouple::new(1, Constraint::PassPoint), ConstraintCouple::new(6, Constraint::PassPoint)];
/// # let constraints = Handle::new(HArray1::from_array(&Array1::from_shared(&ends, 1)));
/// # let mut u = Vector::with_bounds(1, 6)?;
/// # (1..=6).for_each(|i| u[i] = f64::from(i - 1) / 5.0);
/// # let (k, m) = ([0.0, 1.0], [4, 4]);
/// # let (knots, mults) = (Array1::from_shared(&k, 1), Array1::from_shared(&m, 1));
/// let input = BSpGradientInput {
///     ssp: &line,
///     points: [1, 6],
///     constraints: &constraints,
///     knots: &knots,
///     mults: &mults,
///     deg: 3,
///     tolerances: [1e-3, 1e-6],
///     nb_iterations: 1,
/// };
/// let gradient = BSpGradient::new(&input, &mut u)?;
/// assert!(gradient.is_done());
/// assert_eq!(MultiCurveEval::degree(&gradient.value())?, 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BSpGradient {
    scu: MultiBSpCurve,
    par_error: Vector<'static>,
    av_error: f64,
    m_error3d: f64,
    m_error2d: f64,
    lambda1: f64,
    lambda2: f64,
    is_lambda_defined: bool,
    done: bool,
}

/// The inputs shared by the two constructors and Perform.
pub struct BSpGradientInput<'a, L: LineTool + Clone> {
    /// The MultiLine.
    pub ssp: &'a L,
    /// `FirstPoint`, `LastPoint`.
    pub points: [i32; 2],
    /// `TheConstraints`.
    pub constraints: &'a Constraints,
    /// `Knots`.
    pub knots: &'a Array1<'a, f64>,
    /// `Mults`.
    pub mults: &'a Array1<'a, i32>,
    /// `Deg`.
    pub deg: i32,
    /// `Tol3d`, `Tol2d`.
    pub tolerances: [f64; 2],
    /// `NbIterations`.
    pub nb_iterations: i32,
}

impl BSpGradient {
    /// The constructor without lambdas; `BSpGradient::new(&input, &mut u)?`.
    pub fn new<L: LineTool + Clone>(
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
    ) -> Result<Self, Failure> {
        Self::build(input, parameters, None)
    }

    /// The constructor with the lambdas of the tangency extremities;
    /// `BSpGradient::with_lambdas(&input, &mut u, 1.0, 1.0)?`.
    pub fn with_lambdas<L: LineTool + Clone>(
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
        lambda1: f64,
        lambda2: f64,
    ) -> Result<Self, Failure> {
        Self::build(input, parameters, Some([lambda1, lambda2]))
    }

    fn build<L: LineTool + Clone>(
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
        lambdas: Option<[f64; 2]>,
    ) -> Result<Self, Failure> {
        let [lambda1, lambda2] = lambdas.unwrap_or([0., 0.]);
        let mut gradient: Self = Self {
            scu: MultiBSpCurve::new(),
            par_error: Vector::filled(input.points[0], input.points[1], 0.)?,
            av_error: 0.,
            m_error3d: 0.,
            m_error2d: 0.,
            lambda1,
            lambda2,
            is_lambda_defined: lambdas.is_some(),
            done: false,
        };
        gradient.perform(input, parameters)?;
        Ok(gradient)
    }

    /// `Perform(...)` (protected natively).
    fn perform<L: LineTool + Clone>(
        &mut self,
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        self.done = false;
        let [first_point, last_point] = input.points;
        let nbpoles: i32 = -input.deg - 1 + input.mults.iter().sum::<i32>();
        let mut f: BSpFunction<L> = BSpFunction::new(
            input.ssp,
            first_point,
            last_point,
            input.constraints,
            parameters,
            input.knots,
            input.mults,
            nbpoles,
        )?;
        self.set_lambdas(input, parameters, nbpoles, &mut f)?;
        let mut fval: f64 = 0.;
        f.evaluate(parameters, &mut fval)?;
        (self.m_error3d, self.m_error2d) = (f.max_error_3d(), f.max_error_2d());
        self.scu = f.curve_value()?;
        let [tol3d, tol2d] = input.tolerances;
        if self.m_error3d > tol3d || self.m_error2d > tol2d {
            self.project(input, parameters, nbpoles, &f)?;
            f.evaluate(parameters, &mut fval)?;
            (self.m_error3d, self.m_error2d) = (f.max_error_3d(), f.max_error_2d());
        }
        self.optimize(input, parameters, &mut f)
    }

    // The lambdas of the tangency extremities: given, or from a first fit.
    fn set_lambdas<L: LineTool + Clone>(
        &mut self,
        input: &BSpGradientInput<'_, L>,
        parameters: &Vector<'_>,
        nbpoles: i32,
        f: &mut BSpFunction<L>,
    ) -> Result<(), Failure> {
        let [first_point, last_point] = input.points;
        let first_cons: Constraint = point_constraint(input.constraints, first_point)?;
        let last_cons: Constraint = point_constraint(input.constraints, last_point)?;
        if first_cons < Constraint::TangencyPoint && last_cons < Constraint::TangencyPoint {
            return Ok(());
        }
        if self.is_lambda_defined {
            f.set_first_lambda(self.lambda1);
            f.set_last_lambda(self.lambda2);
            return Ok(());
        }
        let fit: LeastSquare = LeastSquare::with_knots(
            input.ssp,
            input.knots,
            input.mults,
            first_point,
            last_point,
            first_cons,
            last_cons,
            parameters,
            nbpoles,
        )?;
        if first_cons >= Constraint::TangencyPoint {
            self.lambda1 = fit.first_lambda();
            f.set_first_lambda(self.lambda1);
        }
        if last_cons >= Constraint::TangencyPoint {
            self.lambda2 = fit.last_lambda();
            f.set_last_lambda(self.lambda2);
        }
        Ok(())
    }

    // The projection step of the inner parameters on the current B-spline poles.
    fn project<L: LineTool + Clone>(
        &self,
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
        nbpoles: i32,
        f: &BSpFunction<L>,
    ) -> Result<(), Failure> {
        let ssp: &L = input.ssp;
        let (nb3d, nb2d) = (ssp.nb_p3d()?, ssp.nb_p2d()?);
        let poles3d: Vec<Array1<'static, Point3>> = (1..=nb3d)
            .map(|k| self.curve_poles_3d(k, nbpoles))
            .collect::<Result<_, _>>()?;
        let poles2d: Vec<Array1<'static, Point2>> = (1..=nb2d)
            .map(|k| self.curve_poles_2d(nb3d + k, nbpoles))
            .collect::<Result<_, _>>()?;
        let (a, da, index): (Matrix<'static>, Matrix<'static>, IntegerVector<'static>) = (
            f.function_matrix()?.clone(),
            f.derivative_function_matrix()?.clone(),
            f.index().clone(),
        );
        let (mut points, mut points2d) = (scratch::<Point3>(nb3d)?, scratch::<Point2>(nb2d)?);
        for j in input.points[0] + 1..=input.points[1] - 1 {
            let u: f64 = *parameters.value(j)?;
            line_points(ssp, j, &mut points, &mut points2d)?;
            let span: [i32; 2] = [*index.value(j)? + 1, *index.value(j)? + 1 + input.deg];
            let (mut fu, mut dfu): (f64, f64) = (0., 0.);
            for (k, poles) in (1..).zip(&poles3d) {
                let [pt, v1] = combine::<3>(&a, &da, j, span, |l| {
                    poles.value(l).map(|p| [p.x(), p.y(), p.z()])
                })?;
                let v1: Vector3 = Vector3::new(v1[0], v1[1], v1[2]);
                fu += Vector3::between(Point3::new(pt[0], pt[1], pt[2]), points[k]).dot(v1);
                dfu += v1.square_magnitude();
            }
            for (k, poles) in (1..).zip(&poles2d) {
                let [pt, v1] =
                    combine::<2>(&a, &da, j, span, |l| poles.value(l).map(|p| [p.x(), p.y()]))?;
                let v1: Vector2 = Vector2::new(v1[0], v1[1]);
                fu += Vector2::between(Point2::new(pt[0], pt[1]), points2d[k]).dot(v1);
                dfu += v1.square_magnitude();
            }
            *parameters.change_value(j)? = projected(u, (fu, dfu));
        }
        Ok(())
    }

    fn curve_poles_3d(&self, k: i32, nbpoles: i32) -> Result<Array1<'static, Point3>, Failure> {
        let mut poles: Array1<'static, Point3> = scratch(nbpoles)?;
        self.scu.curve_3d(k, &mut poles)?;
        Ok(poles)
    }

    fn curve_poles_2d(&self, k: i32, nbpoles: i32) -> Result<Array1<'static, Point2>, Failure> {
        let mut poles: Array1<'static, Point2> = scratch(nbpoles)?;
        self.scu.curve_2d(k, &mut poles)?;
        Ok(poles)
    }

    // BFGS when the tolerances are not reached, then the final curves, parameters
    // and errors.
    fn optimize<L: LineTool + Clone>(
        &mut self,
        input: &BSpGradientInput<'_, L>,
        parameters: &mut Vector<'_>,
        f: &mut BSpFunction<L>,
    ) -> Result<(), Failure> {
        let [tol3d, tol2d] = input.tolerances;
        if self.m_error3d <= tol3d && self.m_error2d <= tol2d {
            self.done = true;
        } else if input.nb_iterations != 0 {
            GradientBfgs::new(f, parameters, tol3d, tol2d, BFGS_EPS, input.nb_iterations)?;
            if let Some(failure) = f.take_failure() {
                return Err(failure);
            }
        }
        self.scu = f.curve_value()?;
        let [first_point, last_point] = input.points;
        let nb_p: i32 = input.ssp.nb_p3d()? + input.ssp.nb_p2d()?;
        self.av_error = 0.;
        for j in first_point..=last_point {
            *parameters.change_value(j)? = *f.new_parameters().value(j)?;
            for k in 1..=nb_p {
                let error: f64 = f.error(j, k)?;
                if self.par_error[j] < error {
                    self.par_error[j] = error;
                }
            }
            self.av_error += self.par_error[j];
        }
        self.av_error /= f64::from(last_point - first_point + 1);
        (self.m_error3d, self.m_error2d) = (f.max_error_3d(), f.max_error_2d());
        if self.m_error3d <= tol3d && self.m_error2d <= tol2d {
            self.done = true;
        }
        Ok(())
    }

    /// `Value()`: the curves; `gradient.value()`.
    pub fn value(&self) -> MultiBSpCurve {
        self.scu.clone()
    }

    /// `IsDone()`; `gradient.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Error(Index)`; `gradient.error(3)?`.
    pub fn error(&self, index: i32) -> Result<f64, Failure> {
        self.par_error.value(index).copied()
    }

    /// `AverageError()`; `gradient.average_error()`.
    pub fn average_error(&self) -> f64 {
        self.av_error
    }

    /// `MaxError3d()`; `gradient.max_error_3d()`.
    pub fn max_error_3d(&self) -> f64 {
        self.m_error3d
    }

    /// `MaxError2d()`; `gradient.max_error_2d()`.
    pub fn max_error_2d(&self) -> f64 {
        self.m_error2d
    }
}

// The point and derivative of one curve at row j of A and DA over the span
// (`a += aa px; d += da px; ...` coordinate by coordinate, as natively).
fn combine<const N: usize>(
    a: &Matrix<'_>,
    da: &Matrix<'_>,
    j: i32,
    [indexdeb, indexfin]: [i32; 2],
    pole: impl Fn(i32) -> Result<[f64; N], rustycad_tkernel::ncollection::ArrayError>,
) -> Result<[[f64; N]; 2], Failure> {
    let (mut point, mut derivative) = ([0.; N], [0.; N]);
    for l in indexdeb..=indexfin {
        let p: [f64; N] = pole(l).map_err(|error| {
            crate::native_failure::array_failure(error, "AppParCurves_BSpGradient")
        })?;
        let (aa, daa) = (*a.value(j, l)?, *da.value(j, l)?);
        for c in 0..N {
            point[c] += aa * p[c];
            derivative[c] += daa * p[c];
        }
    }
    Ok([point, derivative])
}
