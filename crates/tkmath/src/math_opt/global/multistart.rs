//! MathOpt_GlobOpt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
/// Native bounded Powell restarts; `multi_start(&mut f,&lower,&upper,&config)`.
pub fn multi_start(
    function: &mut impl MultipleVarFunction,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    config: &GlobalConfig,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    if !valid(&bounds, config) || config.starts == 0 {
        return failure_result(Status::InvalidInput);
    }
    let mut checked: CheckedFunction<'_, _> = CheckedFunction::new(function);
    let mut rng: RandomGenerator = RandomGenerator::new(config.seed as u64);
    let local: Config =
        config.local(((config.solver.max_iterations as usize / config.starts) as u32).max(10));
    let mut result: VectorResult = VectorResult::default();
    for _ in 0..config.starts {
        let start: Vector<'static> = random_point(&bounds, &mut rng);
        let mut bounded: BoundedFunction<'_, '_, _> =
            BoundedFunction::new(&mut checked, bounds.lower, bounds.upper);
        let solved: VectorResult = powell(&mut bounded, &start, &local);
        let mut candidate: Vector<'static> = match (solved.value, solved.solution) {
            (Some(_), Some(solution)) => solution,
            _ => start,
        };
        bounds.clamp(&mut candidate);
        retain(&mut checked, &mut result, candidate);
    }
    if result.value.is_none() {
        result.status = if checked.failure_status != Status::Ok {
            checked.failure_status
        } else {
            Status::NumericalError
        };
    }
    result
}
fn retain(
    function: &mut impl MultipleVarFunction,
    result: &mut VectorResult,
    candidate: Vector<'static>,
) {
    let mut value: f64 = 0.0;
    if function.value(&candidate, &mut value) && result.value.is_none_or(|old| value < old) {
        result.status = Status::Ok;
        result.solution = Some(candidate);
        result.value = Some(value);
    }
}
