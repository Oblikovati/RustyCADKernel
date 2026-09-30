//! Native global Lipschitz search with Powell, BFGS and Newton local refinement.
use super::{GlobalObjective, MultipleVarFunction, Vector};
use rustycad_tkernel::standard::Failure;
mod candidates;
mod initial;
mod local;
mod parameters;
mod results;
mod search;
/// Global optimizer with explicit derivative capabilities; `GlobalOptimum::new(GlobalObjective::Value(&mut f),&low,&high)?`.
pub struct GlobalOptimum<'a> {
    function: GlobalObjective<'a>,
    dimension: i32,
    lower: Vector<'static>,
    upper: Vector<'static>,
    global_lower: Vector<'static>,
    global_upper: Vector<'static>,
    point: Vector<'static>,
    temporary: Vector<'static>,
    steps: Vector<'static>,
    max_steps: Vector<'static>,
    solutions: Vec<f64>,
    count: i32,
    constant: f64,
    initial_constant: f64,
    locked: bool,
    tolerance: f64,
    same: f64,
    single: bool,
    minimal: f64,
    continuity: i32,
    best: f64,
    done: bool,
    epsilon1: f64,
    epsilon2: f64,
    epsilon3: f64,
    last_step: f64,
    cell_threshold: i32,
}
fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}
impl<'a> GlobalOptimum<'a> {
    /// Uses native C=9, discretization=1e-2, same-point tolerance=1e-7; `GlobalOptimum::new(objective,&low,&high)?`.
    pub fn new(
        function: GlobalObjective<'a>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
    ) -> Result<Self, Failure> {
        Self::with_options(function, lower, upper, 9.0, 1e-2, 1e-7)
    }
    /// Computes initial candidates but leaves Done false; `GlobalOptimum::with_options(objective,&low,&high,9.0,1e-2,1e-7)?`.
    pub fn with_options(
        function: GlobalObjective<'a>,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        constant: f64,
        tolerance: f64,
        same: f64,
    ) -> Result<Self, Failure> {
        let dimension: i32 = function.variables();
        let empty: Vector<'static> = Vector::with_bounds(1, dimension)?;
        let mut result: Self = Self {
            function,
            dimension,
            lower: empty.clone(),
            upper: empty.clone(),
            global_lower: empty.clone(),
            global_upper: empty.clone(),
            point: empty.clone(),
            temporary: empty.clone(),
            steps: empty.clone(),
            max_steps: empty,
            solutions: Vec::new(),
            count: 0,
            constant,
            initial_constant: constant,
            locked: false,
            tolerance,
            same,
            single: false,
            minimal: -2e100,
            continuity: 2,
            best: 2e100,
            done: false,
            epsilon1: 0.0,
            epsilon2: 0.0,
            epsilon3: 0.0,
            last_step: 0.0,
            cell_threshold: (2 * 3.0_f64.powf(f64::from(dimension)) as i32).max(200),
        };
        result.copy_global_bounds(lower, upper)?;
        result.initial_solutions()?;
        Ok(result)
    }
}
