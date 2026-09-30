//! Sign analysis of CSLib_NormalPolyDef over the angular domain of a singular point.
use super::ParametricPosition;
use crate::cslib::NormalPolyDef;
use crate::math::{Function, FunctionRoots};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision::{confusion, p_confusion};
use rustycad_tkernel::standard::Failure;
use std::cmp::Ordering;
use std::f64::consts::PI;

/// Sample count passed to math_FunctionRoots.
const MAX_ROOT_ITERATIONS: i32 = 200;
/// EpsX passed to math_FunctionRoots.
const ROOT_FINDING_TOL: f64 = 1e-5;

/// The angular interval of admissible directions `[inf, sup]`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Domain {
    inf: f64,
    sup: f64,
}

impl Domain {
    /// Interior points use `[-pi, pi]`; bounds and corners restrict it as natively.
    pub(super) fn at(position: ParametricPosition) -> Self {
        let first_u: bool = (position.u - position.u_min).abs() < p_confusion();
        let last_u: bool = (position.u - position.u_max).abs() < p_confusion();
        let first_v: bool = (position.v - position.v_min).abs() < p_confusion();
        let last_v: bool = (position.v - position.v_max).abs() < p_confusion();
        if last_u {
            return Self::last_u(first_v, last_v);
        }
        if first_u {
            return Self::first_u(first_v, last_v);
        }
        if last_v {
            return Self { inf: -PI, sup: 0.0 };
        }
        if first_v {
            return Self { inf: 0.0, sup: PI };
        }
        Self { inf: -PI, sup: PI }
    }
    fn last_u(first_v: bool, last_v: bool) -> Self {
        let mut domain: Self = Self {
            inf: PI / 2.0,
            sup: 3.0 * PI / 2.0,
        };
        if last_v {
            domain.inf = PI;
        }
        if first_v {
            domain.sup = PI;
        }
        domain
    }
    fn first_u(first_v: bool, last_v: bool) -> Self {
        let mut domain: Self = Self {
            inf: -PI / 2.0,
            sup: PI / 2.0,
        };
        if last_v {
            domain.sup = 0.0;
        }
        if first_v {
            domain.inf = 0.0;
        }
        domain
    }
}

/// Returns the polynomial's last sampled value when its sign is constant over
/// `domain`, or `None` when the sign changes (infinitely many normals).
pub(super) fn polynomial_sign(
    order: i32,
    ratios: &[f64],
    domain: Domain,
) -> Result<Option<f64>, Failure> {
    let mut coefficients: Vec<f64> = ratios.to_vec();
    let polynomial_ratios: Array1<'_, f64> = Array1::from_slice(&mut coefficients);
    let mut polynomial: NormalPolyDef = NormalPolyDef::new(order, &polynomial_ratios)?;
    let roots: FunctionRoots = FunctionRoots::with_options(
        &mut polynomial,
        domain.inf,
        domain.sup,
        MAX_ROOT_ITERATIONS,
        [ROOT_FINDING_TOL, confusion(), confusion()],
        0.0,
    );
    let solutions: Vec<f64> = sorted_roots(&roots)?;
    if solutions.is_empty() {
        // No roots: the polynomial keeps its sign, sampled at the lower bound.
        let mut value: f64 = 0.0;
        polynomial.value(domain.inf, &mut value);
        return Ok(Some(value));
    }
    Ok(sign_between_roots(&mut polynomial, &solutions, domain))
}

fn sorted_roots(roots: &FunctionRoots) -> Result<Vec<f64>, Failure> {
    if !roots.is_done() {
        return Ok(Vec::new());
    }
    let count: i32 = roots.number_of_solutions()?;
    let mut solutions: Vec<f64> = (1..=count)
        .map(|index| roots.value(index))
        .collect::<Result<Vec<f64>, Failure>>()?;
    // std::sort by operator<; incomparable values keep an unspecified order natively.
    solutions.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    Ok(solutions)
}

/// Samples the midpoint of every interval between consecutive roots (domain
/// bounds included) longer than PConfusion and tracks sign changes.
///
/// Native quirk, preserved: the first-interval marker `aFirst` stores the
/// interval index, so when the first retained interval is index 0 the marker
/// stays 0 and the next retained interval restarts the comparison, discarding
/// any sign change between those two intervals.
fn sign_between_roots(
    polynomial: &mut NormalPolyDef,
    solutions: &[f64],
    domain: Domain,
) -> Option<f64> {
    let mut bounds: Vec<f64> = Vec::with_capacity(solutions.len() + 2);
    bounds.push(domain.inf);
    bounds.extend_from_slice(solutions);
    bounds.push(domain.sup);
    let mut first: usize = 0;
    let mut changes_sign: bool = false;
    let mut previous: f64 = 0.0;
    let mut next: f64 = 0.0;
    for index in 0..bounds.len() - 1 {
        // Written as the native `>` test so a NaN width also skips the interval.
        let is_wide: bool = (bounds[index + 1] - bounds[index]).abs() > p_confusion();
        if !is_wide {
            continue;
        }
        polynomial.value((bounds[index] + bounds[index + 1]) / 2.0, &mut next);
        if first == 0 {
            first = index;
            changes_sign = false;
        } else {
            changes_sign = changes_sign || (previous * next) < 0.0;
        }
        previous = next;
    }
    if changes_sign { None } else { Some(next) }
}
