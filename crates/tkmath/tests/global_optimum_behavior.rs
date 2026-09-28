//! Observable native constructor and reconfiguration state, including retained candidates.
use rustycad_tkmath::math::{GlobalObjective, GlobalOptimum, MultipleVarFunction, Vector};
struct ConstantObjective(f64);
impl MultipleVarFunction for ConstantObjective {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, _point: &Vector<'_>, value: &mut f64) -> bool {
        *value = self.0;
        true
    }
}
#[test]
fn initial_candidates_are_readable_before_perform() {
    let mut f: ConstantObjective = ConstantObjective(3.0);
    let low: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 1, 1.0).unwrap();
    let solver: GlobalOptimum<'_> =
        GlobalOptimum::new(GlobalObjective::Value(&mut f), &low, &high).unwrap();
    assert!(!solver.is_done());
    assert_eq!(solver.minimum(), 3.0);
    assert_eq!(solver.continuity(), 2);
    assert!(solver.number_of_extrema() > 0);
    assert_eq!(solver.functional_minimal_value(), -2e100);
    let mut point: Vector<'static> = low.clone();
    solver.points(1, &mut point).unwrap();
    assert_eq!(point[1], 0.5);
}
#[test]
fn global_reset_retains_better_old_value_and_accessible_old_sequence() {
    let mut first: ConstantObjective = ConstantObjective(3.0);
    let mut second: ConstantObjective = ConstantObjective(5.0);
    let low: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 1, 1.0).unwrap();
    let mut solver: GlobalOptimum<'_> =
        GlobalOptimum::new(GlobalObjective::Value(&mut first), &low, &high).unwrap();
    solver
        .set_global_params(
            GlobalObjective::Value(&mut second),
            &low,
            &high,
            9.0,
            1e-2,
            1e-7,
        )
        .unwrap();
    assert!(!solver.is_done());
    assert_eq!(solver.number_of_extrema(), 0);
    assert_eq!(solver.minimum(), 3.0);
    let mut point: Vector<'static> = low.clone();
    solver.points(1, &mut point).unwrap();
    assert_eq!(point[1], 0.5);
    assert_eq!(
        solver.points(0, &mut point).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn degenerate_local_search_retains_initial_candidates_and_is_not_done() {
    let mut f: ConstantObjective = ConstantObjective(3.0);
    let low: Vector<'static> = Vector::filled(1, 1, 0.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 1, 1.0).unwrap();
    let mut solver: GlobalOptimum<'_> =
        GlobalOptimum::new(GlobalObjective::Value(&mut f), &low, &high).unwrap();
    let count: i32 = solver.number_of_extrema();
    solver.set_local_params(&low, &low).unwrap();
    solver.perform(false).unwrap();
    assert!(!solver.is_done());
    assert_eq!(solver.number_of_extrema(), count);
    assert_eq!(solver.minimum(), 3.0);
}
struct FlatPlane;
impl MultipleVarFunction for FlatPlane {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, _point: &Vector<'_>, value: &mut f64) -> bool {
        *value = 6.0;
        true
    }
}
#[test]
fn native_cell_filter_branch_keeps_candidates_after_its_threshold() {
    let mut f: FlatPlane = FlatPlane;
    let low: Vector<'static> = Vector::filled(1, 2, -1.0).unwrap();
    let high: Vector<'static> = Vector::filled(1, 2, 2.0).unwrap();
    let mut solver: GlobalOptimum<'_> =
        GlobalOptimum::with_options(GlobalObjective::Value(&mut f), &low, &high, 9.0, 0.03, 1e-7)
            .unwrap();
    solver.set_continuity(0);
    solver.set_lipschitz_locked(true);
    solver.set_functional_minimal_value(6.0);
    solver.perform(false).unwrap();
    assert!(solver.is_done());
    // Exact native count from the persisted global numeric case at OCCT 3d097a0.
    assert_eq!(solver.number_of_extrema(), 1158);
}
