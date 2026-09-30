use super::*;
/// Backtracks using [initial alpha,c1,rho]; `armijo_backtrack(&mut f,&x,&d,&g,fx,[1.0,1e-4,0.5],50)`.
pub fn armijo_backtrack(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    parameters: [f64; 3],
    iterations: u32,
) -> LineSearchResult {
    let [alpha, c1, rho]: [f64; 3] = parameters;
    let mut result: LineSearchResult = LineSearchResult {
        alpha,
        ..Default::default()
    };
    if !common_valid(point, direction, gradient, fx, alpha, iterations)
        || !c1.is_finite()
        || c1 <= 0.0
        || c1 >= 1.0
        || !rho.is_finite()
        || rho <= 0.0
        || rho >= 1.0
    {
        result.status = Status::InvalidInput;
        return result;
    }
    let derivative: f64 = match directional(point, direction, gradient) {
        Ok(value) => value,
        Err(status) => {
            result.status = status;
            return result;
        }
    };
    let mut candidate: Vector<'_> = Vector::with_size(point.length() as usize);
    for iteration in 0..iterations {
        if !new_point(point, direction, result.alpha, &mut candidate) {
            result.status = Status::NumericalError;
            return result;
        }
        let Some(value) = result.evaluate(f, &candidate) else {
            return result;
        };
        result.f_new = value;
        if !result.finite(value) {
            return result;
        }
        if value <= fx + c1 * result.alpha * derivative {
            result.success();
            return result;
        }
        let next: f64 = result.alpha * rho;
        if next < f64::EPSILON || iteration + 1 == iterations {
            break;
        }
        result.alpha = next;
    }
    result.status = Status::MaxIterations;
    result
}
