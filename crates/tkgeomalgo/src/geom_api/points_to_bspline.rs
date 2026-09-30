// Rust translation of GeomAPI_PointsToBSpline.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_PointsToBSpline`: a B-spline curve approximating 3D points.
use crate::geom2d_api::points_to_bspline::{continuity_code, has_meaningful_span};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeombase::app_def::{
    BSplineCompute, MultiLine, MultiPointConstraint, Variational, VariationalOptions,
};
use rustycad_tkgeombase::app_par_curves::{
    Constraint, ConstraintCouple, MultiBSpCurve, MultiCurveEval,
};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "GeomAPI_PointsToBSpline";

// A fresh native array.
fn array<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| e.into_failure(MEMBER))
}

// The value at `index` of a native array (Standard_OutOfRange outside it).
fn at<T: Copy>(values: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    values
        .value(index)
        .copied()
        .map_err(|e| e.into_failure(MEMBER))
}

/// `GeomAPI_PointsToBSpline`: the B-spline of an AppDef_BSplineCompute least squares
/// approximation (or of an AppDef_Variational smoothing), with the degree,
/// continuity and tolerance asked. Each constructor calls the Init of the same
/// arguments; the exceptions of AppDef and Geom are returned as `Err`.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::PointsToBSpline;
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::gp::Point3;
/// let points: Vec<Point3> = (0..8).map(|i| Point3::new(f64::from(i), f64::from(i * i) / 10.0, 0.5)).collect();
/// let fit = PointsToBSpline::with_points(&Array1::from_shared(&points, 1), 3, 8, Shape::C2, 1e-3).unwrap();
/// assert!(fit.is_done());
/// ```
#[derive(Default)]
pub struct PointsToBSpline {
    my_is_done: bool,
    my_curve: Handle<GeomBSplineCurve>,
}

impl PointsToBSpline {
    /// `GeomAPI_PointsToBSpline()`: not done. `PointsToBSpline::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_PointsToBSpline(Points, DegMin, DegMax, Continuity, Tol3D)` (defaults
    /// 3, 8, C2, 1e-3). `PointsToBSpline::with_points(&points, 3, 8, Shape::C2, 1e-3)?`.
    pub fn with_points(
        points: &Array1<'_, Point3>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init(points, deg_min, deg_max, continuity, tol3d)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSpline(Points, ParType, DegMin, DegMax, Continuity, Tol3D)`.
    /// `PointsToBSpline::with_parametrization(&points, ParametrizationType::Centripetal,
    /// 3, 8, Shape::C2, 1e-3)?`.
    pub fn with_parametrization(
        points: &Array1<'_, Point3>,
        par_type: ParametrizationType,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_parametrization(points, par_type, deg_min, deg_max, continuity, tol3d)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSpline(Points, Parameters, DegMin, DegMax, Continuity,
    /// Tol3D)`. `PointsToBSpline::with_parameters(&points, &params, 3, 8, Shape::C2,
    /// 1e-3)?`.
    pub fn with_parameters(
        points: &Array1<'_, Point3>,
        params: &Array1<'_, f64>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_parameters(points, params, deg_min, deg_max, continuity, tol3d)?;
        Ok(fit)
    }
    /// `GeomAPI_PointsToBSpline(Points, Weight1, Weight2, Weight3, DegMax, Continuity,
    /// Tol3D)`: the variational smoothing.
    /// `PointsToBSpline::with_weights(&points, (1.0, 0.0, 0.0), 8, Shape::C2, 1e-3)?`.
    pub fn with_weights(
        points: &Array1<'_, Point3>,
        weights: (f64, f64, f64),
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::new();
        fit.init_weights(points, weights, deg_max, continuity, tol3d)?;
        Ok(fit)
    }

    /// `Init(Points, DegMin, DegMax, Continuity, Tol3D)`: chord-length
    /// parametrization. `fit.init(&points, 3, 8, Shape::C2, 1e-3)?`.
    pub fn init(
        &mut self,
        points: &Array1<'_, Point3>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.init_parametrization(
            points,
            ParametrizationType::ChordLength,
            deg_min,
            deg_max,
            continuity,
            tol3d,
        )
    }
    /// `Init(Points, ParType, DegMin, DegMax, Continuity, Tol3D)`: two BFGS
    /// iterations, a plain least-squares fit when `Tol3D <= 1e-3`.
    /// `fit.init_parametrization(&points, ParametrizationType::ChordLength, 3, 8,
    /// Shape::C2, 1e-3)?`.
    pub fn init_parametrization(
        &mut self,
        points: &Array1<'_, Point3>,
        par_type: ParametrizationType,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let options = ComputeLineOptions {
            degree_min: deg_min,
            degree_max: deg_max,
            tolerance_3d: tol3d,
            tolerance_2d: 0.0,
            nb_iterations: 2,
            cutting: true,
            parametrization: par_type,
            squares: tol3d <= 1.0e-3,
        };
        let mut the_computer = BSplineCompute::new(options);
        the_computer.set_continuity(continuity_code(continuity));
        the_computer.perform(&MultiLine::from_points(points))?;
        let the_curve: &MultiBSpCurve = the_computer.value();
        let knots: Array1<'static, f64> = the_curve.knots()?;
        self.set_curve(the_curve, &knots)
    }
    /// `Init(Points, Parameters, DegMin, DegMax, Continuity, Tol3D)`: the parameters
    /// rescaled to `[0, 1]` for the fit, and the knots mapped back onto the first and
    /// last parameters; Standard_OutOfRange when the counts differ, not done (without
    /// exception) when the parameter span is not meaningful.
    /// `fit.init_parameters(&points, &params, 3, 8, Shape::C2, 1e-3)?`.
    pub fn init_parameters(
        &mut self,
        points: &Array1<'_, Point3>,
        params: &Array1<'_, f64>,
        deg_min: i32,
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        if params.len() != points.len() {
            return Err(Failure::out_of_range(
                "GeomAPI_PointsToBSpline::Init() - invalid input",
            ));
        }
        let Some(the_params) = unit_parameters(params)? else {
            return Ok(());
        };
        let options = ComputeLineOptions {
            degree_min: deg_min,
            degree_max: deg_max,
            tolerance_3d: tol3d,
            tolerance_2d: 0.0,
            nb_iterations: 0,
            cutting: true,
            parametrization: ParametrizationType::IsoParametric,
            squares: true,
        };
        let mut the_computer = BSplineCompute::new(options);
        the_computer.set_parameters(&the_params)?;
        the_computer.set_continuity(continuity_code(continuity));
        the_computer.perform(&MultiLine::from_points(points))?;
        let the_curve: &MultiBSpCurve = the_computer.value();
        let mut knots: Array1<'static, f64> = the_curve.knots()?;
        let (first, last) = (at(params, params.lower())?, at(params, params.upper())?);
        bsplclib::reparametrize(first, last, &mut knots)?;
        self.set_curve(the_curve, &knots)
    }

    // The Geom_BSplineCurve of the first curve of an approximation, on `knots`.
    fn set_curve(
        &mut self,
        the_curve: &MultiBSpCurve,
        knots: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut poles: Array1<'static, Point3> = array(1, the_curve.nb_poles())?;
        the_curve.curve_3d(1, &mut poles)?;
        let mults = the_curve.multiplicities()?;
        let curve = GeomBSplineCurve::new(&poles, knots, &mults, the_curve.degree()?, false)?;
        self.my_curve = Handle::new(curve);
        self.my_is_done = true;
        Ok(())
    }

    /// `Curve()`: StdFail_NotDone when the approximation failed. `fit.curve()?`.
    pub fn curve(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        if !self.my_is_done {
            return Err(crate::geom2d_api::not_done(
                "GeomAPI_PointsToBSpline::Curve",
            ));
        }
        Ok(self.my_curve.clone())
    }
    /// `operator occ::handle<Geom_BSplineCurve>()`: the curve, null when never done.
    /// `fit.as_curve()`.
    pub fn as_curve(&self) -> Handle<GeomBSplineCurve> {
        self.my_curve.clone()
    }
    /// `IsDone()`. `fit.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }
}

// The parameters rescaled to `[0, 1]` (None when their span is not meaningful). The
// native code reads `Params(i)` for i in `2..Nbp-1` whatever the lower bound of Params.
fn unit_parameters(params: &Array1<'_, f64>) -> Result<Option<Vector<'static>>, Failure> {
    let nbp: i32 = params.len() as i32;
    let mut the_params: Vector<'static> = Vector::with_bounds(1, nbp)?;
    the_params[1] = 0.0;
    the_params[nbp] = 1.0;
    let uf: f64 = at(params, params.lower())?;
    let ul: f64 = at(params, params.upper())? - uf;
    if !has_meaningful_span(ul) {
        return Ok(None);
    }
    for i in 2..nbp {
        the_params[i] = (at(params, i)? - uf) / ul;
    }
    Ok(Some(the_params))
}

impl PointsToBSpline {
    /// `Init(Points, Weight1, Weight2, Weight3, DegMax, Continuity, Tol3D)`: the
    /// AppDef_Variational smoothing (1000 segments, 2 iterations, none when
    /// `Tol3D <= 1e-3`); not done when it is not created, over-constrained, fails (its
    /// exception is swallowed, as natively) or not done.
    /// `fit.init_weights(&points, (1.0, 0.0, 0.0), 8, Shape::C2, 1e-3)?`.
    pub fn init_weights(
        &mut self,
        points: &Array1<'_, Point3>,
        (w1, w2, w3): (f64, f64, f64),
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let nb_point: i32 = points.len() as i32;
        let nbit: i32 = if tol3d <= 1.0e-3 { 0 } else { 2 };
        let mult_l = MultiLine::with_nb_multi_points(nb_point)?;
        for i in 1..=nb_point {
            let mpc = MultiPointConstraint::with_counts(1, 0)?;
            mpc.set_point(1, at(points, points.lower() + i - 1)?)?;
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
        variation.set_tolerance(tol3d)?;
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
        let knots: Array1<'static, f64> = the_curve.knots()?;
        self.set_curve(&the_curve, &knots)
    }
}
