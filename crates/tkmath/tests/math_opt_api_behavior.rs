//! Supplemental behavior of the public `MathOpt::Utils` helpers and the modern Uzawa solver
//! (not an upstream suite; the differential probe `mathopt_*` pins the exact bits).
use rustycad_tkmath::math::{
    Function, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
use rustycad_tkmath::math_opt::utils::*;
use rustycad_tkmath::math_opt::{HessianFunction, UzawaConfig, uzawa, uzawa_equality};
use rustycad_tkmath::math_utils::{Config, LineSearchResult, Status};

/// `(x0 - 1)^2 + x1^2 + saddle * x0 * x1`, refusing the `fail`-th callback.
struct Bowl {
    saddle: f64,
    fail: usize,
    calls: usize,
}

impl Bowl {
    fn new(saddle: f64, fail: usize) -> Self {
        Self {
            saddle,
            fail,
            calls: 0,
        }
    }

    fn accept(&mut self) -> bool {
        self.calls += 1;
        self.calls != self.fail
    }
}

impl MultipleVarFunction for Bowl {
    fn variables(&self) -> i32 {
        2
    }

    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        let (a, b): (f64, f64) = (*x.at(0).unwrap(), *x.at(1).unwrap());
        *f = (a - 1.0) * (a - 1.0) + b * b + self.saddle * a * b;
        self.accept()
    }
}

impl MultipleVarFunctionWithGradient for Bowl {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        let (a, b): (f64, f64) = (*x.at(0).unwrap(), *x.at(1).unwrap());
        *g.change_at(0).unwrap() = 2.0 * (a - 1.0) + self.saddle * b;
        *g.change_at(1).unwrap() = 2.0 * b + self.saddle * a;
        self.accept()
    }

    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        MultipleVarFunction::value(self, x, f) && self.gradient(x, g)
    }
}

impl HessianFunction for Bowl {
    fn hessian(&mut self, _x: &Vector<'_>, h: &mut Matrix<'_>) -> bool {
        *h.change_at(0, 0).unwrap() = f64::INFINITY;
        self.accept()
    }
}

impl Function for Bowl {
    fn value(&mut self, x: f64, f: &mut f64) -> bool {
        *f = 1.0 / x;
        self.accept()
    }
}

fn point(a: f64, b: f64) -> Vector<'static> {
    let mut v: Vector<'static> = Vector::with_size(2);
    *v.change_at(0).unwrap() = a;
    *v.change_at(1).unwrap() = b;
    v
}

#[test]
fn mathopt_api_validation_predicates() {
    let mut config: Config = Config::new(1e-8, 10);
    config.relative_tolerance = 0.0;
    assert!(is_valid_config(&config));
    config.step_min = 0.0;
    assert!(!is_valid_config(&config));
    assert!(!is_finite_vector(&Vector::with_size(0)));
    assert!(!is_finite_vector(&point(1.0, f64::NAN)));
    let mut m: Matrix<'static> = Matrix::filled(0, 1, 0, 1, 1.0).unwrap();
    assert!(is_finite_matrix(&m));
    *m.change_at(1, 1).unwrap() = f64::NEG_INFINITY;
    assert!(!is_finite_matrix(&m));
    assert!(is_valid_bounds(&point(-1.0, 0.0), &point(1.0, 2.0)));
    assert!(!is_valid_bounds(&point(-1.0, 2.0), &point(1.0, 2.0)));
    assert!(!is_valid_bounds(
        &point(-f64::MAX, 0.0),
        &point(f64::MAX, 1.0)
    ));
    assert_eq!(norm(&point(3e300, 4e300)), 5e300);
    assert_eq!(FRPR_WOLFE_CURVATURE, 0.1);
}

#[test]
fn mathopt_api_evaluation_statuses() {
    let mut value: f64 = 0.0;
    let mut bowl: Bowl = Bowl::new(0.0, 2);
    assert_eq!(
        scalar_value_status(&mut bowl, 0.0, &mut value),
        Status::NumericalError
    );
    assert_eq!(
        scalar_value_status(&mut bowl, 1.0, &mut value),
        Status::CallbackError
    );
    assert_eq!(
        value_status(&mut bowl, &point(f64::NAN, 0.0), &mut value),
        Status::NumericalError
    );
    assert!(checked_value(&mut bowl, &point(1.0, 2.0), &mut value));
    assert_eq!(value, 4.0);
    let mut g: Vector<'static> = Vector::with_size(2);
    assert_eq!(
        gradient_status(&mut bowl, &point(2.0, 1.0), &mut g),
        Status::Ok
    );
    assert!(checked_gradient(&mut bowl, &point(2.0, 1.0), &mut g));
    assert_eq!((*g.at(0).unwrap(), *g.at(1).unwrap()), (2.0, 2.0));
    let mut h: Matrix<'static> = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    assert_eq!(
        hessian_status(&mut bowl, &point(0.0, 0.0), &mut h),
        Status::NumericalError
    );
    assert!(!checked_hessian(&mut bowl, &point(0.0, 0.0), &mut h));
}

#[test]
fn mathopt_api_checked_and_bounded_adapters() {
    let mut bowl: Bowl = Bowl::new(0.0, 2);
    let mut value: f64 = 0.0;
    {
        let mut checked = CheckedFunction::new(&mut bowl);
        assert!(checked.value(&point(1.0, 0.0), &mut value));
        assert!(!checked.value(&point(1.0, 0.0), &mut value));
        assert!(checked.value(&point(0.0, 0.0), &mut value));
        assert!(checked.had_failure && checked.has_valid_evaluation);
        assert_eq!(checked.failure_status, Status::CallbackError);
        assert_eq!(checked.original().calls, 3);
    }
    let (lower, upper): (Vector<'static>, Vector<'static>) = (point(-1.0, -1.0), point(0.5, 0.5));
    let mut bounded = BoundedFunction::new(&mut bowl, &lower, &upper);
    assert!(bounded.value(&point(9.0, f64::NAN), &mut value));
    // x0 clamps to 0.5 and the NaN coordinate to the upper bound, as native max(lo,min(hi,x)).
    assert_eq!(value, 0.25 + 0.25);
}

#[test]
fn mathopt_api_line_search_statuses_and_backtrack() {
    let mut failed: LineSearchResult = LineSearchResult::default();
    assert_eq!(line_search_failure_status(&failed), Status::NotConverged);
    failed.status = Status::NumericalError;
    assert!(is_line_search_evaluation_error(&failed));
    let mut fallback: LineSearchResult = LineSearchResult::default();
    assert_eq!(
        line_search_fallback_failure_status(&failed, &fallback),
        Status::NumericalError
    );
    fallback.status = Status::CallbackError;
    assert_eq!(
        line_search_fallback_failure_status(&failed, &fallback),
        Status::CallbackError
    );
    let mut bowl: Bowl = Bowl::new(0.0, 0);
    let x: Vector<'static> = point(3.0, 0.0);
    let g: Vector<'static> = point(4.0, 0.0);
    let step: LineSearchResult = backtrack(&mut bowl, &x, &point(-4.0, 0.0), &g, 4.0, 0.5, 1e-12);
    assert!(step.is_valid);
    assert_eq!((step.alpha, step.f_new), (0.5, 0.0));
    let ascent: LineSearchResult = backtrack(&mut bowl, &x, &g, &g, 4.0, 1.0, 1e-12);
    assert!(!ascent.is_valid);
    assert_eq!(ascent.evaluations, 0);
}

#[test]
fn mathopt_api_stationarity_and_hessian_shift() {
    let config: Config = Config::new(1e-8, 100);
    let mut status: Status = Status::MaxIterations;
    assert!(is_stationary(
        &mut Bowl::new(0.0, 0),
        &point(1.0, 0.0),
        0.0,
        &config,
        &mut status
    ));
    assert_eq!(status, Status::Ok);
    // A saddle passes every coordinate probe; only the diagonal probe exposes descent.
    let mut saddle: Bowl = Bowl::new(4.0, 0);
    assert!(!is_stationary(
        &mut saddle,
        &point(0.0, 0.0),
        1.0,
        &config,
        &mut status
    ));
    assert_eq!(status, Status::Ok);
    assert!(!is_stationary(
        &mut Bowl::new(0.0, 1),
        &point(1.0, 0.0),
        0.0,
        &config,
        &mut status
    ));
    assert_eq!(status, Status::CallbackError);
    let mut h: Matrix<'static> = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
    *h.change_at(0, 0).unwrap() = -2.0;
    *h.change_at(1, 1).unwrap() = 3.0;
    assert!(regularize_hessian(&mut h, 1.0));
    assert_eq!((*h.at(0, 0).unwrap(), *h.at(1, 1).unwrap()), (1.0, 6.0));
}

#[test]
fn mathopt_api_uzawa_equality_and_inequality() {
    let config: UzawaConfig = UzawaConfig::default();
    let c: Matrix<'static> = Matrix::filled(0, 0, 0, 1, 1.0).unwrap();
    let s: Vector<'static> = Vector::filled(0, 0, 2.0).unwrap();
    let direct = uzawa_equality(&c, &s, &point(0.0, 0.0), &config);
    assert!(direct.is_done() && bool::from(&direct));
    assert_eq!(direct.iterations, 1);
    assert_eq!(*direct.solution.as_ref().unwrap().at(1).unwrap(), 1.0);
    assert_eq!(*direct.inverse_ctc.as_ref().unwrap().at(0, 0).unwrap(), 0.5);
    // x0 + x1 <= 2 is inactive at the origin: the multiplier projects to zero.
    let inactive = uzawa(&c, &s, &point(0.0, 0.0), 0, 1, &config);
    assert_eq!(inactive.status, Status::Ok);
    assert_eq!(*inactive.dual.as_ref().unwrap().at(0).unwrap(), 0.0);
    assert!(inactive.inverse_ctc.is_none());
    let active = uzawa(&c, &s, &point(3.0, 3.0), 0, 1, &config);
    assert!(active.is_done());
    assert!((*active.solution.as_ref().unwrap().at(0).unwrap() - 1.0).abs() <= 1e-5);
    let wrong = uzawa(&c, &s, &point(0.0, 0.0), 1, 1, &config);
    assert_eq!(wrong.status, Status::InvalidInput);
    let zero: Matrix<'static> = Matrix::filled(0, 0, 0, 1, 0.0).unwrap();
    assert_eq!(
        uzawa(&zero, &s, &point(0.0, 0.0), 1, 0, &config).status,
        Status::Singular
    );
    assert_eq!(
        uzawa(&zero, &s, &point(0.0, 0.0), 0, 1, &config).status,
        Status::Singular
    );
}
