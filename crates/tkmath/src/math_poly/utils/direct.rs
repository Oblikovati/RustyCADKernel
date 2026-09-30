//! `SolveDirect`: closed-form linear and quadratic roots, long-double isolation above.
use super::{
    LongCoefficients, derivative, long_coefficients, root_multiplicity_tolerance, scaled,
    small_residual, solve_distinct_real, to_double, validate,
};
use crate::math_poly::PolyResult;
use crate::math_poly::extended::{LongDouble, exponent, scale_double};
use crate::math_utils::Status;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl;
/// `SolveDirect`: real roots with multiplicities of `c[0] + c[1] x + ... + c[degree] x^degree`
/// after trailing zero coefficients are dropped. Degree 0 reports `InfiniteSolutions` or
/// `NoSolution`; a failure keeps the roots already appended. Above degree 4 a fifth distinct
/// real root overruns the native four-root buffer; the port raises Standard_OutOfRange
/// instead (degree 4 and below never reach it); `utils::solve_direct(&[-6.0, 11.0, -6.0, 1.0], 3)?`.
pub fn solve_direct(coefficients: &[f64], degree: i32) -> Result<PolyResult, Failure> {
    if !validate(Some(coefficients), degree, 1.0) {
        return Ok(PolyResult::failed(Status::InvalidInput));
    }
    let mut degree: i32 = degree;
    while degree > 0 && coefficients[degree as usize] == 0.0 {
        degree -= 1;
    }
    Ok(match degree {
        0 => PolyResult::failed(if coefficients[0] == 0.0 {
            Status::InfiniteSolutions
        } else {
            Status::NoSolution
        }),
        1 => linear_result(coefficients),
        2 => quadratic_result(coefficients),
        _ => higher_result(coefficients, degree)?,
    })
}
fn linear_result(c: &[f64]) -> PolyResult {
    let Some(root) = to_double(-LongDouble::new(c[0]) / LongDouble::new(c[1])) else {
        return PolyResult::failed(Status::NumericalError);
    };
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    out.append(root, 1);
    out
}
/// Scales by the largest coefficient's binary exponent and uses the FMA-compensated
/// discriminant `b^2 - 4ac` of the native source.
fn quadratic_result(c: &[f64]) -> PolyResult {
    let maximum: f64 = c[..3].iter().fold(0.0_f64, |s, x| s.max(x.abs()));
    let power: i32 = exponent(maximum);
    let a: f64 = scale_double(c[2], -power);
    let b: f64 = scale_double(c[1], -power);
    let c: f64 = scale_double(c[0], -power);
    let four_a: f64 = 4.0 * a;
    let four_ac: f64 = four_a * c;
    let product_error: f64 = (-four_a).mul_add(c, four_ac);
    let discriminant: f64 = b.mul_add(b, -four_ac) + product_error;
    if discriminant < 0.0 {
        return PolyResult::failed(Status::Ok);
    }
    if discriminant == 0.0 {
        return repeated_quadratic(-b / (2.0 * a));
    }
    let q: f64 = -0.5 * (b + discriminant.sqrt().copysign(b));
    distinct_quadratic(q / a, c / q)
}
fn repeated_quadratic(root: f64) -> PolyResult {
    if !root.is_finite() {
        return PolyResult::failed(Status::NumericalError);
    }
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    out.append(root, 2);
    out
}
fn distinct_quadratic(first: f64, second: f64) -> PolyResult {
    let mut out: PolyResult = PolyResult::failed(Status::Ok);
    for root in [first, second] {
        if !root.is_finite() {
            out.status = Status::NumericalError;
            out.nb_roots = 0;
            return out;
        }
        out.append(root, 1);
    }
    if out.roots[0] > out.roots[1] {
        out.roots.swap(0, 1);
    }
    out
}
fn higher_result(c: &[f64], degree: i32) -> Result<PolyResult, Failure> {
    // Native code ignores the result; the trimmed leading coefficient is nonzero anyway.
    let (scaled, power) = scaled(c, degree).unwrap_or((long_coefficients(), 0));
    let mut roots: [LongDouble; 4] = [LongDouble::ZERO; 4];
    let mut count: usize = 0;
    if !solve_distinct_real(&scaled, degree, &mut roots, &mut count)? {
        return Ok(PolyResult::failed(Status::NumericalError));
    }
    stl::sort_by(&mut roots[..count], |a, b| a < b);
    let mut out: PolyResult = PolyResult::failed(Status::NotConverged);
    let final_tolerance: LongDouble = LongDouble::new(256.0) * LongDouble::epsilon();
    for &root in &roots[..count] {
        let multiplicity: usize = multiplicity(&scaled, degree, root);
        let value: Option<f64> = to_double(root.scale(power));
        let (true, Some(value)) = (
            small_residual(&scaled, degree, root, final_tolerance),
            value,
        ) else {
            out.status = Status::NumericalError;
            return Ok(out);
        };
        out.append(value, multiplicity);
    }
    out.status = Status::Ok;
    Ok(out)
}
/// Counts how many successive derivatives also vanish at `root`.
fn multiplicity(scaled: &LongCoefficients, degree: i32, root: LongDouble) -> usize {
    let mut current: LongCoefficients = *scaled;
    let mut current_degree: i32 = degree;
    let mut count: usize = 1;
    while count < degree as usize && current_degree > 0 {
        let next: LongCoefficients = derivative(&current, current_degree);
        current_degree -= 1;
        if !small_residual(&next, current_degree, root, root_multiplicity_tolerance()) {
            break;
        }
        count += 1;
        current = next;
    }
    count
}
