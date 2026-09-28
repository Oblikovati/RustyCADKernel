//! Regression coverage for native stopping order, callback state and result guards.
use rustycad_tkmath::math::{
    Bfgs, Frpr, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
struct StatefulObjective {
    states: i32,
    fail_value: bool,
    fail_combined: bool,
}
impl MultipleVarFunction for StatefulObjective {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = point[1] * point[1];
        !self.fail_value
    }
    fn state_number(&mut self) -> i32 {
        self.states += 1;
        self.states
    }
}
impl MultipleVarFunctionWithGradient for StatefulObjective {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * point[1];
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        *value = point[1] * point[1];
        self.gradient(point, gradient);
        !self.fail_combined
    }
}
#[test]
fn bfgs_preserves_convergence_before_failed_direction_and_does_not_capture_state() {
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: true,
        fail_combined: false,
    };
    let mut solver: Bfgs = Bfgs::new(1).unwrap();
    solver
        .perform(&mut function, &Vector::filled(1, 1, 2.0).unwrap())
        .unwrap();
    assert!(solver.is_done());
    assert!(solver.is_solution_reached());
    assert_eq!(solver.minimum().unwrap(), 4.0);
    assert_eq!(solver.location().unwrap()[1], 2.0);
    assert_eq!(solver.gradient().unwrap()[1], 4.0);
    assert_eq!(function.states, 0);
}
#[test]
fn frpr_captures_state_once_at_stationary_start() {
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: false,
        fail_combined: false,
    };
    let mut solver: Frpr = Frpr::new(&function, 1e-8).unwrap();
    solver
        .perform(&mut function, &Vector::filled(1, 1, 0.0).unwrap())
        .unwrap();
    assert!(solver.is_done());
    assert!(solver.is_solution_reached());
    assert_eq!(solver.location().unwrap()[1], 0.0);
    assert_eq!(solver.minimum().unwrap(), 0.0);
    assert_eq!(function.states, 1);
}
macro_rules! check_guards {
    ($solver:expr) => {{
        let solver = $solver;
        let mut output: Vector<'static> = Vector::filled(-3, -3, 71.0).unwrap();
        assert!(!solver.is_done());
        assert!(solver.location().is_err());
        assert!(solver.gradient().is_err());
        assert!(solver.minimum().is_err());
        assert!(solver.iterations().is_err());
        assert!(solver.copy_location(&mut output).is_err());
        assert!(solver.copy_gradient(&mut output).is_err());
        assert_eq!(output[-3], 71.0);
    }};
}
#[test]
fn unperformed_and_failed_optimizers_guard_all_results() {
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: false,
        fail_combined: true,
    };
    let mut bfgs: Bfgs = Bfgs::new(1).unwrap();
    let mut frpr: Frpr = Frpr::new(&function, 1e-8).unwrap();
    check_guards!(&bfgs);
    check_guards!(&frpr);
    let start: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    bfgs.perform(&mut function, &start).unwrap();
    frpr.perform(&mut function, &start).unwrap();
    check_guards!(&bfgs);
    check_guards!(&frpr);
    assert_eq!(function.states, 0);
}
impl rustycad_tkmath::math::MultipleVarFunctionWithHessian for StatefulObjective {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut rustycad_tkmath::math::Matrix<'_>,
    ) -> bool {
        self.values(point, value, gradient);
        hessian[(1, 1)] = 2.0;
        !self.fail_combined
    }
}
#[test]
fn newton_done_is_independent_of_iteration_limit_status() {
    use rustycad_tkmath::math::{NewtonMinimum, Status};
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: false,
        fail_combined: false,
    };
    let mut solver: NewtonMinimum =
        NewtonMinimum::with_options(&function, 1e-8, 0, 1e-6, true).unwrap();
    assert_eq!(solver.status(), Status::NotBracketed);
    solver
        .perform(&mut function, &Vector::filled(1, 1, 2.0).unwrap())
        .unwrap();
    assert!(solver.is_done());
    assert_eq!(solver.status(), Status::TooManyIterations);
    assert_eq!(solver.iterations().unwrap(), 1);
    assert_eq!(solver.location().unwrap()[1], 0.0);
    assert_eq!(solver.gradient().unwrap()[1], 4.0);
    assert_eq!(function.states, 0);
}
#[test]
fn newton_failed_hessian_guards_results_and_preserves_status_access() {
    use rustycad_tkmath::math::{NewtonMinimum, Status};
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: false,
        fail_combined: true,
    };
    let mut solver: NewtonMinimum = NewtonMinimum::new(&function).unwrap();
    check_guards!(&solver);
    solver
        .perform(&mut function, &Vector::filled(1, 1, 2.0).unwrap())
        .unwrap();
    check_guards!(&solver);
    assert_eq!(solver.status(), Status::FunctionError);
    assert_eq!(
        [
            Status::Ok as i32,
            Status::TooManyIterations as i32,
            Status::FunctionError as i32,
            Status::DirectionSearchError as i32,
            Status::NotBracketed as i32
        ],
        [0, 1, 2, 3, 4]
    );
}
#[test]
fn powell_captures_state_and_clears_success_before_a_failed_search() {
    use rustycad_tkmath::math::{Matrix, Powell};
    let mut function: StatefulObjective = StatefulObjective {
        states: 0,
        fail_value: false,
        fail_combined: false,
    };
    let mut solver: Powell = Powell::new(&function, 1e-8).unwrap();
    let start: Vector<'static> = Vector::filled(1, 1, 2.0).unwrap();
    let direction: Matrix<'static> = Matrix::filled(1, 1, 1, 1, 1.0).unwrap();
    solver.perform(&mut function, &start, &direction).unwrap();
    assert!(solver.is_done());
    assert!(solver.is_solution_reached());
    assert_eq!(function.states, 1);
    function.fail_value = true;
    solver.perform(&mut function, &start, &direction).unwrap();
    assert!(!solver.is_done());
    assert_eq!(function.states, 1);
    assert!(solver.minimum().is_err());
    assert!(solver.iterations().is_err());
    assert!(solver.location().is_err());
    let mut output: Vector<'static> = Vector::filled(-3, -3, 71.0).unwrap();
    assert!(solver.copy_location(&mut output).is_err());
    assert_eq!(output[-3], 71.0);
}
