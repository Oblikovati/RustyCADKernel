//! Convergence hooks, status bookkeeping, Dump text and the small math value types.
use rustycad_tkmath::math::{
    Bfgs, BissecNewton, BrentMinimum, DirectPolynomialRoots, Frpr, Function, FunctionSet,
    FunctionSetRoot, FunctionSetWithDerivatives, FunctionWithDerivative, GaussPointsAndWeights,
    GaussSetIntegration, GaussSingleIntegration, IntegerVector, KronrodPointsAndWeights, Matrix,
    MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    NewtonFunctionSetRoot, NewtonMinimum, Powell, PsoParticle, PsoParticlesPool, Status,
    TrigonometricRoots, ValueAndWeight, Vector,
};

/// f(x) = sum (x_i - i)^2.
struct Bowl;
impl MultipleVarFunction for Bowl {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, v: &mut f64) -> bool {
        *v = (x[1] - 1.0).powi(2) + (x[2] - 2.0).powi(2);
        true
    }
}
impl MultipleVarFunctionWithGradient for Bowl {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        g[1] = 2.0 * (x[1] - 1.0);
        g[2] = 2.0 * (x[2] - 2.0);
        true
    }
    fn values(&mut self, x: &Vector<'_>, v: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, v) && self.gradient(x, g)
    }
}
impl MultipleVarFunctionWithHessian for Bowl {
    fn values_hessian(
        &mut self,
        x: &Vector<'_>,
        v: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> bool {
        h.init(0.0);
        h.set_diag(2.0).unwrap();
        self.values(x, v, g)
    }
}
fn origin() -> Vector<'static> {
    Vector::filled(1, 2, 0.0).unwrap()
}
fn text(write: impl FnOnce(&mut String) -> std::fmt::Result) -> String {
    let mut out = String::new();
    write(&mut out).unwrap();
    out
}

#[test]
fn gradient_solvers_expose_status_state_and_dump() {
    let mut bfgs = Bfgs::new(2).unwrap();
    assert_eq!(bfgs.status(), Status::Ok);
    bfgs.perform(&mut Bowl, &origin()).unwrap();
    assert!(bfgs.is_done());
    assert_eq!(bfgs.status(), Status::Ok);
    assert!(bfgs.current_minimum() <= bfgs.previous_minimum());
    assert_eq!((bfgs.tolerance(), bfgs.zero_epsilon()), (1e-8, 1e-12));
    assert_eq!(bfgs.current_location().length(), 2);
    assert_eq!(bfgs.current_gradient().length(), 2);
    assert!(bfgs.to_string().starts_with(
        "math_BFGS resolution:  Status = Done \n Location Vector = math_Vector of Length = 2\n"
    ));
    let mut frpr = Frpr::with_options(&Bowl, 1e-8, 1, 1e-12).unwrap();
    assert_eq!(frpr.status(), Status::NotBracketed);
    frpr.perform_with_criterion(&mut Bowl, &origin(), &mut |_, _| false)
        .unwrap();
    assert_eq!(frpr.status(), Status::TooManyIterations);
    assert_eq!(
        text(|t| frpr.dump(t)),
        "math_FRPR  Status = not Done because 1\n"
    );
}
#[test]
fn criteria_replace_the_virtual_tests() {
    let mut tests = 0;
    let mut bfgs = Bfgs::new(2).unwrap();
    bfgs.perform_with_criterion(&mut Bowl, &origin(), &mut |solver: &Bfgs, _: &mut Bowl| {
        tests += 1;
        solver.current_minimum() < 10.0
    })
    .unwrap();
    assert_eq!((tests, bfgs.iterations().unwrap()), (1, 1));
    let mut directions = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    directions.set_diag(1.0).unwrap();
    let mut powell = Powell::new(&Bowl, 1e-8).unwrap();
    powell
        .perform_with_criterion(&mut Bowl, &origin(), &directions, &mut |_, _| true)
        .unwrap();
    assert_eq!(powell.iterations().unwrap(), 1);
    assert!(
        powell
            .to_string()
            .starts_with("math_Powell resolution: Status = Done")
    );
    let mut newton = NewtonMinimum::new(&Bowl).unwrap();
    let mut checks = 0;
    newton
        .perform_with_criterion(&mut Bowl, &origin(), &mut |solver: &NewtonMinimum| {
            checks += 1;
            solver.current_step().length() == 2
        })
        .unwrap();
    assert_eq!((checks, newton.iterations().unwrap()), (2, 2));
}
#[test]
fn newton_minimum_dump_raises_after_status_lines_when_not_done() {
    let newton = NewtonMinimum::new(&Bowl).unwrap();
    let mut out = String::new();
    let error = newton.dump(&mut out).unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    assert_eq!(
        out,
        "math_Newton Optimisation:  Done   =0\n Status = 4\n Location Vector = "
    );
}
struct Cubic;
impl Function for Cubic {
    fn value(&mut self, x: f64, v: &mut f64) -> bool {
        *v = x * x * x - 8.0;
        true
    }
}
impl FunctionWithDerivative for Cubic {
    fn derivative(&mut self, x: f64, d: &mut f64) -> bool {
        *d = 3.0 * x * x;
        true
    }
    fn values(&mut self, x: f64, v: &mut f64, d: &mut f64) -> bool {
        self.value(x, v) && self.derivative(x, d)
    }
}
#[test]
fn scalar_hooks_accessors_and_dumps() {
    let mut brent = BrentMinimum::new(1e-10);
    let mut parabola = |x: f64, v: &mut f64| {
        *v = (x - 1.0) * (x - 1.0);
        true
    };
    brent.perform_with_criterion(&mut parabola, [0.0, 0.5, 3.0], &mut |solver, _| {
        solver.bracket()[1] - solver.bracket()[0] < 10.0
    });
    assert_eq!(brent.iterations().unwrap(), 1);
    assert_eq!((brent.tolerance(), brent.zero_epsilon()), (1e-10, 1e-12));
    assert!(brent.current_value() >= 0.0 && brent.current_location() == 0.5);
    let mut bissec = BissecNewton::new(1e-10);
    assert_eq!(bissec.status(), Status::NotBracketed);
    bissec.perform(&mut Cubic, 0.0, 4.0, 100);
    assert_eq!(bissec.status(), Status::Ok);
    assert!((bissec.current_root() - 2.0).abs() < 1e-9 && bissec.current_step().abs() <= 1e-10);
    assert_eq!(bissec.tolerance(), 1e-10);
    assert!(
        bissec
            .to_string()
            .starts_with("math_BissecNewton  Status = Done \n The Root  is: 2\n")
    );
    bissec.perform(&mut Cubic, 3.0, 4.0, 100);
    assert_eq!(bissec.status(), Status::NotBracketed);
    assert_eq!(
        bissec.to_string(),
        "math_BissecNewton  Status = not Done \n"
    );
}
#[test]
fn polynomial_dumps_follow_native_text() {
    assert_eq!(
        DirectPolynomialRoots::quadratic(1.0, -3.0, 2.0).to_string(),
        "math_DirectPolynomialRoots  Status = Not Infinity Roots \n Number of solutions = 2\n Solution number 1 = 2\n Solution number 2 = 1\n"
    );
    assert_eq!(
        DirectPolynomialRoots::linear(0.0, 0.0).to_string(),
        "math_DirectPolynomialRoots  Status = Infinity Roots \n"
    );
    let roots = TrigonometricRoots::sine(1.0, 0.0, -1.0, 1.0);
    assert!(
        roots
            .to_string()
            .starts_with(" math_TrigonometricFunctionRoots: \n Number of solutions = 1\n")
    );
}
/// F_i(x) = x_i^2 - 4.
struct Squares;
impl FunctionSet for Squares {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, r: &mut Vector<'_>) -> bool {
        r[1] = x[1] * x[1] - 4.0;
        true
    }
}
impl FunctionSetWithDerivatives for Squares {
    fn derivatives(&mut self, x: &Vector<'_>, m: &mut Matrix<'_>) -> bool {
        m[(1, 1)] = 2.0 * x[1];
        true
    }
    fn values(&mut self, x: &Vector<'_>, r: &mut Vector<'_>, m: &mut Matrix<'_>) -> bool {
        self.value(x, r) && self.derivatives(x, m)
    }
}
#[test]
fn set_solvers_accept_criteria_and_dump() {
    let tolerance = Vector::filled(1, 1, 1e-10).unwrap();
    let start = Vector::filled(1, 1, 3.0).unwrap();
    let bounds = [
        &Vector::filled(1, 1, -f64::MAX).unwrap(),
        &Vector::filled(1, 1, f64::MAX).unwrap(),
    ];
    let mut root = FunctionSetRoot::new(&Squares, &tolerance, 50).unwrap();
    root.perform(&mut Squares, &start, false).unwrap();
    assert!(
        root.to_string()
            .starts_with(" math_FunctionSetRoot Status = Done\n Location value = ")
    );
    assert_eq!(root.tolerances()[1], 1e-10);
    assert_eq!(root.current_jacobian().row_number(), 1);
    assert!(root.current_step()[1].abs() <= 1e-10 && root.current_solution()[1] == 2.0);
    let mut newton = NewtonFunctionSetRoot::new(&Squares, &tolerance, 1e-10, 50).unwrap();
    let mut tests = 0;
    newton
        .perform_with_criterion(&mut Squares, &start, bounds, &mut |solver, _| {
            tests += 1;
            solver.current_values()[1].abs() < 1.0
        })
        .unwrap();
    assert_eq!((tests, newton.iterations().unwrap()), (1, 1));
    assert_eq!(
        (newton.tolerances()[1], newton.function_tolerance()),
        (1e-10, 1e-10)
    );
    assert!(newton.current_step()[1] < 0.0);
    assert!(
        newton
            .to_string()
            .contains(" Value of the function at this solution = \nmath_Vector")
    );
}
/// [t^2, 3t] over one variable.
struct Curve(i32);
impl FunctionSet for Curve {
    fn variables(&self) -> i32 {
        self.0
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, r: &mut Vector<'_>) -> bool {
        r[1] = x[1] * x[1];
        r[2] = 3.0 * x[1];
        true
    }
}
#[test]
fn gauss_set_integration_integrates_each_equation() {
    let lower = Vector::filled(1, 1, 0.0).unwrap();
    let upper = Vector::filled(1, 1, 2.0).unwrap();
    let order = IntegerVector::filled(1, 1, 4).unwrap();
    let integral = GaussSetIntegration::new(&mut Curve(1), &lower, &upper, &order).unwrap();
    let value = integral.value().unwrap();
    assert!((value[1] - 8.0 / 3.0).abs() < 1e-14 && (value[2] - 6.0).abs() < 1e-14);
    assert!(
        integral
            .to_dump()
            .starts_with("math_GaussSetIntegration  Status = Done \nIntegration Value = ")
    );
    let error = GaussSetIntegration::new(&mut Curve(2), &lower, &upper, &order)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(error.message().contains("2 variables"));
    let high = IntegerVector::filled(1, 1, 62).unwrap();
    assert!(GaussSetIntegration::new(&mut Curve(1), &lower, &upper, &high).is_err());
}
trait ToDump {
    fn to_dump(&self) -> String;
}
impl ToDump for GaussSetIntegration {
    fn to_dump(&self) -> String {
        text(|t| self.dump(t))
    }
}
#[test]
fn single_integration_default_and_dump() {
    let empty = GaussSingleIntegration::default();
    assert!(!empty.is_done());
    assert_eq!(
        empty.to_string(),
        "math_GaussSingleIntegration Status = not Done \n"
    );
    let mut line = |x: f64, v: &mut f64| {
        *v = x;
        true
    };
    let integral = GaussSingleIntegration::new(&mut line, 0.0, 2.0, 3);
    assert_eq!(
        integral.to_string(),
        "math_GaussSingleIntegration  Status = Done \nIntegration Value = 2\n"
    );
}
#[test]
fn value_and_weight_orders_by_value_only() {
    let mut nodes = vec![
        ValueAndWeight::new(0.5, 1.0),
        ValueAndWeight::new(-0.5, 2.0),
        ValueAndWeight::new(0.0, 3.0),
    ];
    assert!(nodes[1].less(&nodes[0]) && !nodes[0].less(&nodes[1]));
    ValueAndWeight::sort(&mut nodes);
    assert_eq!(
        nodes.iter().map(|n| n.weight()).collect::<Vec<_>>(),
        [2.0, 3.0, 1.0]
    );
    let zero = ValueAndWeight::default();
    assert_eq!((zero.value(), zero.weight()), (0.0, 0.0));
}
#[test]
fn quadrature_orders_follow_native_edges() {
    let gauss = GaussPointsAndWeights::new(0).unwrap();
    assert!(!gauss.is_done());
    assert_eq!(gauss.points().length(), 0);
    assert!(GaussPointsAndWeights::new(-1).is_err());
    assert!(KronrodPointsAndWeights::new(0).is_err());
    assert_eq!(
        KronrodPointsAndWeights::new(1).unwrap().points().length(),
        3
    );
}
#[test]
fn particles_default_order_and_pool_edges() {
    let particle = PsoParticle::default();
    assert_eq!(
        (particle.distance, particle.best_distance),
        (f64::MAX, f64::MAX)
    );
    assert!(particle.position.is_empty());
    let closer = PsoParticle {
        distance: 1.0,
        ..Default::default()
    };
    assert!(closer.less(&particle) && !particle.less(&closer));
    let mut empty = PsoParticlesPool::new(0, 3).unwrap();
    assert_eq!(
        empty.best_particle().err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        PsoParticlesPool::new(2, 0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        PsoParticlesPool::new(2, -1).err().unwrap().exception_type(),
        "Standard_RangeError"
    );
    assert_eq!(
        PsoParticlesPool::new(-1, 2).err().unwrap().exception_type(),
        "Standard_RangeError"
    );
}
#[test]
fn remaining_solver_dumps_start_with_native_labels() {
    use rustycad_tkmath::math::{
        BracketMinimum, BracketedRoot, Crout, FunctionAllRoots, FunctionRoot, FunctionRoots,
        FunctionSample, Householder, NewtonFunctionRoot, Uzawa,
    };
    let mut parabola = |x: f64, v: &mut f64| {
        *v = (x - 1.0) * (x - 1.0);
        true
    };
    let bracket = BracketMinimum::compute(&mut parabola, 0.0, 0.5);
    assert!(
        bracket
            .to_string()
            .starts_with("math_BracketMinimum  Status = Done \n The bracketed triplet is: \n")
    );
    let bracketed = BracketedRoot::new(&mut Cubic, 0.0, 4.0, 1e-10);
    assert!(bracketed.to_string().contains(" The Root is: 2\n"));
    let root = FunctionRoot::new(&mut Cubic, 3.0, 1e-10).unwrap();
    assert!(root.to_string().ends_with("The value at the root is: 0\n"));
    let newton = NewtonFunctionRoot::new(&mut Cubic, 3.0, 1e-10, 1e-10);
    assert!(newton.to_string().contains(" Location found = 2\n"));
    let roots = FunctionRoots::new(&mut Cubic, 0.0, 4.0, 10);
    assert!(roots.to_string().contains(" Solution Number 1= 2\n"));
    let all =
        FunctionAllRoots::new(&mut Cubic, &FunctionSample::new(0.0, 4.0, 10), [1e-8; 3]).unwrap();
    assert!(
        all.to_string()
            .ends_with("Number of points where the function is null: 1\n")
    );
    let brent = {
        let mut solver = BrentMinimum::new(1e-10);
        solver.perform(&mut parabola, 0.0, 0.5, 3.0);
        solver
    };
    assert!(
        brent
            .to_string()
            .starts_with("math_BrentMinimum  Status = Done \n Location value = 1\n")
    );
    let mut a = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    a.set_diag(2.0).unwrap();
    let b = Vector::filled(1, 2, 1.0).unwrap();
    assert_eq!(
        Crout::new(&a).unwrap().to_string(),
        "math_Crout  Status = Done \n"
    );
    assert_eq!(
        Householder::from_vector(&a, &b, 1e-20).unwrap().to_string(),
        "math_Householder  Status = Done \n"
    );
    let constraints = Matrix::filled(1, 1, 1, 2, 1.0).unwrap();
    let second = Vector::filled(1, 1, 1.0).unwrap();
    let uzawa = Uzawa::new(&constraints, &second, &b).unwrap();
    assert!(
        uzawa
            .to_string()
            .starts_with("math_Uzawa Status = Done \n Number of iterations = ")
    );
}
