// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of Geom2dAPI_PointsToBSpline.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAPI_PointsToBSpline`: a B-spline curve approximating 2D points.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkgeombase::app_def::{
    BSplineCompute, MultiLine, MultiPointConstraint, Variational, VariationalOptions,
};
use rustycad_tkgeombase::app_par_curves::{
    Constraint, ConstraintCouple, MultiBSpCurve, MultiCurveEval,
};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{self, Point2};
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "Geom2dAPI_PointsToBSpline";

/// `hasMeaningfulSpan(theSpan)`: the square of the span exceeds that of
/// gp::Resolution().
fn has_meaningful_span(the_span: f64) -> bool {
    let a_resolution: f64 = gp::resolution();
    the_span * the_span > a_resolution * a_resolution
}

/// The continuity code of AppDef_BSplineCompute::SetContinuity for a GeomAbs shape.
fn continuity_code(continuity: Shape) -> i32 {
    match continuity {
        Shape::C0 => 0,
        Shape::G1 | Shape::C1 => 1,
        Shape::G2 | Shape::C2 => 2,
        _ => 3,
    }
}

// A fresh native array.
fn array<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| e.into_failure(MEMBER))
}

/// `Geom2dAPI_PointsToBSpline`: the B-spline of an AppDef_BSplineCompute least
/// squares approximation (or of an AppDef_Variational smoothing), with the degree,
/// continuity and tolerance asked. Each constructor calls the Init of the same
/// arguments; the exceptions of AppDef and Geom2d are returned as `Err`.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_api::PointsToBSpline;
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::gp::Point2;
/// let points: Vec<Point2> = (0..8).map(|i| Point2::new(f64::from(i), f64::from(i * i) / 10.0)).collect();
/// let fit = PointsToBSpline::with_points(&Array1::from_shared(&points, 1), 3, 8, Shape::C2, 1e-6).unwrap();
/// assert!(fit.is_done());
/// ```
#[derive(Default)]
pub struct PointsToBSpline {
    my_is_done: bool,
    my_curve: Handle<Geom2dBSplineCurve>,
}

impl PointsToBSpline {
    /// `Geom2dAPI_PointsToBSpline()`: not done. `PointsToBSpline::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Geom2dAPI_PointsToBSpline(Points, DegMin, DegMax, Continuity, Tol2D)`
    /// (defaults 3, 8, C2, 1e-6). `PointsToBSpline::with_points(&points, 3, 8, Shape::C2, 1e-6)?`.
    pub fn with_points(
        points: &Array1<'_, Point2>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init(points, deg_min, deg_max, continuity, tol2d)?;
        Ok(fit)
    }
    /// `Geom2dAPI_PointsToBSpline(YValues, X0, DX, DegMin, DegMax, Continuity, Tol2D)`.
    /// `PointsToBSpline::with_y_values(&y, 0.0, 1.0, 3, 8, Shape::C2, 1e-6)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_y_values(
        y_values: &Array1<'_, f64>,
        x0: f64,
        dx: f64,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_y_values(y_values, x0, dx, deg_min, deg_max, continuity, tol2d)?;
        Ok(fit)
    }
    /// `Geom2dAPI_PointsToBSpline(Points, ParType, DegMin, DegMax, Continuity, Tol2D)`
    /// (defaults 3, 8, C2, 1e-3).
    /// `PointsToBSpline::with_parametrization(&points, ParametrizationType::Centripetal, 3, 8, Shape::C2, 1e-3)?`.
    pub fn with_parametrization(
        points: &Array1<'_, Point2>,
        par_type: ParametrizationType,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_parametrization(points, par_type, deg_min, deg_max, continuity, tol2d)?;
        Ok(fit)
    }
    /// `Geom2dAPI_PointsToBSpline(Points, Parameters, DegMin, DegMax, Continuity, Tol2D)`.
    /// `PointsToBSpline::with_parameters(&points, &params, 3, 8, Shape::C2, 1e-3)?`.
    pub fn with_parameters(
        points: &Array1<'_, Point2>,
        params: &Array1<'_, f64>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_parameters(points, params, deg_min, deg_max, continuity, tol2d)?;
        Ok(fit)
    }
    /// `Geom2dAPI_PointsToBSpline(Points, Weight1, Weight2, Weight3, DegMax,
    /// Continuity, Tol3D)`: the variational smoothing.
    /// `PointsToBSpline::with_weights(&points, 1.0, 0.0, 0.0, 8, Shape::C2, 1e-3)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_weights(
        points: &Array1<'_, Point2>,
        w1: f64,
        w2: f64,
        w3: f64,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_weights(points, (w1, w2, w3), deg_max, continuity, tol2d)?;
        Ok(fit)
    }

    /// `Init(Points, DegMin, DegMax, Continuity, Tol2D)`: chord-length
    /// parametrization. `fit.init(&points, 3, 8, Shape::C2, 1e-6)?`.
    pub fn init(
        &mut self,
        points: &Array1<'_, Point2>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.init_parametrization(
            points,
            ParametrizationType::ChordLength,
            deg_min,
            deg_max,
            continuity,
            tol2d,
        )
    }

    /// `Init(Points, ParType, DegMin, DegMax, Continuity, Tol2D)`: two BFGS
    /// iterations, a plain least-squares fit when `Tol2D <= 1e-3`.
    /// `fit.init_parametrization(&points, ParametrizationType::ChordLength, 3, 8, Shape::C2, 1e-3)?`.
    pub fn init_parametrization(
        &mut self,
        points: &Array1<'_, Point2>,
        par_type: ParametrizationType,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let options = ComputeLineOptions {
            degree_min: deg_min,
            degree_max: deg_max,
            tolerance_3d: 0.0,
            tolerance_2d: tol2d,
            nb_iterations: 2,
            cutting: true,
            parametrization: par_type,
            squares: tol2d <= 1.0e-3,
        };
        let mut the_computer = BSplineCompute::new(options);
        the_computer.set_continuity(continuity_code(continuity));
        the_computer.perform(&MultiLine::from_points2d(points))?;
        self.set_curve(the_computer.value())
    }

    /// `Init(Points, Parameters, DegMin, DegMax, Continuity, Tol2D)`: the parameters
    /// rescaled to `[0, 1]`; Standard_OutOfRange when the counts differ, not done
    /// (without exception) when the parameter span is not meaningful.
    /// `fit.init_parameters(&points, &params, 3, 8, Shape::C2, 1e-3)?`.
    pub fn init_parameters(
        &mut self,
        points: &Array1<'_, Point2>,
        params: &Array1<'_, f64>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        if params.len() != points.len() {
            return Err(Failure::out_of_range(
                "Geom2dAPI_PointsToBSpline::Init() - invalid input",
            ));
        }
        let nbp: i32 = params.len() as i32;
        let mut the_params: Vector<'static> = Vector::with_bounds(1, nbp)?;
        the_params[1] = 0.0;
        the_params[nbp] = 1.0;
        let uf: f64 = *params
            .value(params.lower())
            .map_err(|e| e.into_failure(MEMBER))?;
        let ul: f64 = *params
            .value(params.upper())
            .map_err(|e| e.into_failure(MEMBER))?
            - uf;
        if !has_meaningful_span(ul) {
            return Ok(());
        }
        for i in 2..nbp {
            // The native code reads Params(i) whatever the lower bound of Params.
            the_params[i] = (*params.value(i).map_err(|e| e.into_failure(MEMBER))? - uf) / ul;
        }
        let options = ComputeLineOptions {
            degree_min: deg_min,
            degree_max: deg_max,
            tolerance_3d: 0.0,
            tolerance_2d: tol2d,
            nb_iterations: 0,
            cutting: true,
            parametrization: ParametrizationType::IsoParametric,
            squares: true,
        };
        let mut the_computer = BSplineCompute::new(options);
        the_computer.set_parameters(&the_params)?;
        the_computer.set_continuity(continuity_code(continuity));
        the_computer.perform(&MultiLine::from_points2d(points))?;
        self.set_curve(the_computer.value())
    }

    // The Geom2d_BSplineCurve of the first curve of an approximation.
    fn set_curve(&mut self, the_curve: &MultiBSpCurve) -> Result<(), Failure> {
        let mut poles: Array1<'static, Point2> = array(1, the_curve.nb_poles())?;
        the_curve.curve_2d(1, &mut poles)?;
        let knots = the_curve.knots()?;
        let mults = the_curve.multiplicities()?;
        let curve = Geom2dBSplineCurve::new(&poles, &knots, &mults, the_curve.degree()?, false)?;
        self.my_curve = Handle::new(curve);
        self.my_is_done = true;
        Ok(())
    }

    /// `Curve()`: StdFail_NotDone when the approximation failed. `fit.curve()?`.
    pub fn curve(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        if !self.my_is_done {
            return Err(super::not_done("Geom2dAPI_PointsToBSpline::Curve"));
        }
        Ok(self.my_curve.clone())
    }
    /// `operator occ::handle<Geom2d_BSplineCurve>()`: the curve, null when never
    /// done. `fit.as_curve()`.
    pub fn as_curve(&self) -> Handle<Geom2dBSplineCurve> {
        self.my_curve.clone()
    }
    /// `IsDone()`. `fit.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }
}

impl PointsToBSpline {
    /// `Init(Points, Weight1, Weight2, Weight3, DegMax, Continuity, Tol2D)`: the
    /// AppDef_Variational smoothing (1000 segments, 2 iterations, none when
    /// `Tol2D <= 1e-3`); not done when it is not created, over-constrained, fails
    /// (its exception is swallowed, as natively) or not done.
    /// `fit.init_weights(&points, (1.0, 0.0, 0.0), 8, Shape::C2, 1e-3)?`.
    pub fn init_weights(
        &mut self,
        points: &Array1<'_, Point2>,
        (w1, w2, w3): (f64, f64, f64),
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let nb_point: i32 = points.len() as i32;
        let nbit: i32 = if tol2d <= 1.0e-3 { 0 } else { 2 };
        let mult_l = MultiLine::with_nb_multi_points(nb_point)?;
        for i in 1..=nb_point {
            let mpc = MultiPointConstraint::with_counts(0, 1)?;
            let point: Point2 = *points
                .value(points.lower() + i - 1)
                .map_err(|e| e.into_failure(MEMBER))?;
            mpc.set_point2d(1, point)?;
            mult_l.set_value(i, &mpc)?;
        }
        let mut couples: Array1<'static, ConstraintCouple> = array(1, nb_point)?;
        for i in 1..=nb_point {
            couples[i] = ConstraintCouple::new(i, Constraint::NoConstraint);
        }
        let tab_of_cc = Handle::new(HArray1::from_owned(couples));
        let mut variation = Variational::new(
            &mult_l,
            1,
            nb_point,
            &tab_of_cc,
            VariationalOptions::default(),
        )?;
        variation.set_max_degree(deg_max)?;
        variation.set_continuity(continuity)?;
        variation.set_max_segment(1000);
        variation.set_tolerance(tol2d)?;
        variation.set_with_min_max(false)?;
        variation.set_nb_iterations(nbit);
        variation.set_criterium_weight(w1, w2, w3)?;
        if !variation.is_created() || variation.is_over_constrained() {
            return Ok(());
        }
        // A failure of the approximation leaves the result not done.
        if variation.approximate().is_err() || !variation.is_done() {
            return Ok(());
        }
        let the_curve: MultiBSpCurve = variation.value()?;
        self.set_curve(&the_curve)
    }

    /// `Init(YValues, X0, DX, DegMin, DegMax, Continuity, Tol2D)`: Y fitted over
    /// parameters proportional to X (uniform when `X0 + length` is not meaningful),
    /// then X set from a line raised to the degree with the knots inserted, so that
    /// `X(t) = t` on `[X0, X0 + DX * (n - 1)]`.
    /// `fit.init_y_values(&y, 0.0, 1.0, 3, 8, Shape::C2, 1e-6)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn init_y_values(
        &mut self,
        y_values: &Array1<'_, f64>,
        x0: f64,
        dx: f64,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol2d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let (lower, upper) = (y_values.lower(), y_values.upper());
        let length: f64 = dx * f64::from(upper - lower);
        let (points, param) = super::points_to_bspline_y::y_samples(y_values, x0, (dx, length))?;
        let options = ComputeLineOptions {
            degree_min: deg_min,
            degree_max: deg_max,
            tolerance_3d: 0.0,
            tolerance_2d: tol2d,
            nb_iterations: 0,
            cutting: true,
            parametrization: ParametrizationType::IsoParametric,
            squares: true,
        };
        let mut the_computer = BSplineCompute::from_parameters(&param, options)?;
        the_computer.set_continuity(continuity_code(continuity));
        the_computer.perform(&MultiLine::from_points2d(&points))?;
        let curve = super::points_to_bspline_y::x_as_parameter(the_computer.value(), x0, length)?;
        self.my_curve = Handle::new(curve);
        self.my_is_done = true;
        Ok(())
    }
}
