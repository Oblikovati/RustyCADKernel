//! `AppendUnique` and `SolveDistinctReal`: real-root isolation between critical points.
use super::preconditions::{fixed_degree, free_slot, stored_count};
use super::{
    DISTINCT_ROOT_BISECTION_ITERATIONS, LongCoefficients, cauchy_bound, derivative,
    distinct_root_residual_tolerance, horner, small_residual,
};
use crate::math_poly::extended::LongDouble;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl;
/// The native `std::array<long double, 4>` root buffers.
const ROOT_SLOTS: usize = 4;
/// `AppendUnique`: appends `root` unless a stored root lies within
/// `128 * LDBL_EPSILON * max(1, |root|)`. A fifth distinct root (or a count above 4)
/// overruns the native array; the port raises Standard_OutOfRange instead;
/// `utils::append_unique(&mut roots, &mut count, root)?`.
pub fn append_unique(
    roots: &mut [LongDouble; 4],
    count: &mut usize,
    root: LongDouble,
) -> Result<(), Failure> {
    const MEMBER: &str = "MathPoly::Utils::AppendUnique";
    stored_count(MEMBER, *count, ROOT_SLOTS, "roots")?;
    let tolerance: LongDouble =
        LongDouble::new(128.0) * LongDouble::epsilon() * LongDouble::new(1.0).max(root.abs());
    if roots[..*count]
        .iter()
        .any(|&other| (other - root).abs() <= tolerance)
    {
        return Ok(());
    }
    free_slot(MEMBER, *count, ROOT_SLOTS, "distinct roots")?;
    roots[*count] = root;
    *count += 1;
    Ok(())
}
/// `SolveDistinctReal`: appends the distinct real roots of a degree >= 1 polynomial to
/// `roots[count..]`, recursing on the derivative to bracket them between critical points
/// and the Cauchy bound, then bisecting 200 times. Returns false only for a non-finite
/// linear root. A degree below 1 never terminates natively and returns false here.
/// Standard_OutOfRange above `MAX_POLY_DEGREE` or when more than four distinct roots (at
/// any recursion level) would overrun the native four-root buffers, which a degree of
/// five or more can reach; `utils::solve_distinct_real(&c, 3, &mut roots, &mut count)?`.
pub fn solve_distinct_real(
    coefficients: &LongCoefficients,
    degree: i32,
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) -> Result<bool, Failure> {
    if degree < 1 {
        return Ok(false);
    }
    fixed_degree("MathPoly::Utils::SolveDistinctReal", degree, 1)?;
    if degree == 1 {
        let root: LongDouble = -coefficients[0] / coefficients[1];
        if !root.is_finite() {
            return Ok(false);
        }
        append_unique(roots, count, root)?;
        return Ok(true);
    }
    let mut critical: [LongDouble; 4] = [LongDouble::ZERO; 4];
    let mut critical_count: usize = 0;
    let derived: LongCoefficients = derivative(coefficients, degree);
    if !solve_distinct_real(&derived, degree - 1, &mut critical, &mut critical_count)? {
        return Ok(false);
    }
    stl::sort_by(&mut critical[..critical_count], |a, b| a < b);
    let (points, point_count) = interval_points(
        coefficients,
        degree,
        &critical[..critical_count],
        roots,
        count,
    )?;
    bracketed_roots(coefficients, degree, &points[..point_count], roots, count)?;
    Ok(true)
}
/// `-bound`, the distinct critical points and `bound`; critical points that are roots
/// themselves are appended on the way.
fn interval_points(
    c: &LongCoefficients,
    degree: i32,
    critical: &[LongDouble],
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) -> Result<([LongDouble; 6], usize), Failure> {
    let mut points: [LongDouble; 6] = [LongDouble::ZERO; 6];
    let bound: LongDouble = cauchy_bound(c, degree);
    points[0] = -bound;
    let mut point_count: usize = 1;
    for (i, &point) in critical.iter().enumerate() {
        if i == 0 || point != critical[i - 1] {
            points[point_count] = point;
            point_count += 1;
        }
        if small_residual(c, degree, point, distinct_root_residual_tolerance()) {
            append_unique(roots, count, point)?;
        }
    }
    points[point_count] = bound;
    Ok((points, point_count + 1))
}
/// Bisects every sign-changing interval whose inner end points are not roots already.
fn bracketed_roots(
    c: &LongCoefficients,
    degree: i32,
    points: &[LongDouble],
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) -> Result<(), Failure> {
    let small = |x: LongDouble| small_residual(c, degree, x, distinct_root_residual_tolerance());
    for i in 1..points.len() {
        let (left, right) = (points[i - 1], points[i]);
        let left_value: LongDouble = horner(c, degree, left).0;
        let right_value: LongDouble = horner(c, degree, right).0;
        if (i > 1 && small(left)) || (i + 1 < points.len() && small(right)) {
            continue;
        }
        if (left_value < LongDouble::ZERO) == (right_value < LongDouble::ZERO) {
            continue;
        }
        append_unique(roots, count, bisected(c, degree, left, right, left_value))?;
    }
    Ok(())
}
fn bisected(
    c: &LongCoefficients,
    degree: i32,
    mut left: LongDouble,
    mut right: LongDouble,
    mut left_value: LongDouble,
) -> LongDouble {
    let two: LongDouble = LongDouble::new(2.0);
    for _ in 0..DISTINCT_ROOT_BISECTION_ITERATIONS {
        let middle: LongDouble = (left + right) / two;
        let value: LongDouble = horner(c, degree, middle).0;
        if (left_value < LongDouble::ZERO) == (value < LongDouble::ZERO) {
            left = middle;
            left_value = value;
        } else {
            right = middle;
        }
    }
    (left + right) / two
}
