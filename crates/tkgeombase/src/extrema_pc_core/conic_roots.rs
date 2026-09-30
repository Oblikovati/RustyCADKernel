// Rust translation of the root loops of performCore in ExtremaPC2d_Hyperbola.hxx,
// ExtremaPC2d_Parabola.hxx, ExtremaPC_Hyperbola.hxx and ExtremaPC_Parabola.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extrema of the hyperbola and parabola evaluators from the real roots of their
//! stationary polynomial. The polynomial has a positive leading coefficient, so the
//! number of odd-multiplicity roots after a root decides whether the stationary
//! function crosses from negative to positive there (a minimum). The hyperbola solves
//! in `v = e^u` and keeps only positive roots; the parabola solves in `u` directly.
use super::result::{PcExtremum, PcResult, SearchMode, Status};
use super::space::PcPoint;
use super::validity::THE_PARAM_TOLERANCE;
use rustycad_tkmath::math_poly::PolyResult;
use rustycad_tkmath::math_utils::{Domain1D, Status as PolyStatus};

/// The polynomial roots and how they map to curve parameters.
pub(crate) struct PolynomialRoots<'r> {
    roots: &'r PolyResult,
    exponential: bool,
}

impl<'r> PolynomialRoots<'r> {
    /// Roots in `v = e^u` (the hyperbola): only `v > 0` maps to `u = ln(v)`.
    pub fn exponential(roots: &'r PolyResult) -> Self {
        Self {
            roots,
            exponential: true,
        }
    }
    /// Roots in `u` (the parabola).
    pub fn direct(roots: &'r PolyResult) -> Self {
        Self {
            roots,
            exponential: false,
        }
    }
    // The curve parameter of root `index`, `None` for a non-positive exponential root.
    fn parameter(&self, index: usize) -> Option<f64> {
        let root: f64 = self.roots.roots[index];
        if !self.exponential {
            return Some(root);
        }
        (root > 0.0).then(|| root.ln())
    }
    // Whether root `index` takes part in the sign count after a root.
    fn counts(&self, index: usize) -> bool {
        !self.exponential || self.roots.roots[index] > 0.0
    }
    // The odd-multiplicity roots after `index`: an even count means a minimum.
    fn is_minimum(&self, index: usize) -> bool {
        let odd_after: usize = (index + 1..self.roots.nb_roots)
            .filter(|&next| self.counts(next))
            .map(|next| self.roots.multiplicities[next] % 2)
            .sum();
        odd_after.is_multiple_of(2)
    }
}

/// The root loop of `performCore`, after the input checks: an unsolved polynomial gives
/// InfiniteSolutions (the distance at parameter 0) or NumericalError, otherwise every
/// admissible root of odd multiplicity is an extremum.
pub(crate) fn add_polynomial_extrema<P: PcPoint>(
    result: &mut PcResult<P>,
    roots: &PolynomialRoots<'_>,
    (point, domain, mode): (&P, Option<&Domain1D>, SearchMode),
    value: impl Fn(f64) -> P,
) {
    if !roots.roots.is_done() {
        if roots.roots.status == PolyStatus::InfiniteSolutions {
            result.status = Status::InfiniteSolutions;
            result.infinite_square_distance = point.square_distance_to(&value(0.0));
        } else {
            result.status = Status::NumericalError;
        }
        return;
    }
    for index in 0..roots.roots.nb_roots {
        let Some(mut u) = roots.parameter(index) else {
            continue;
        };
        if let Some(domain) = domain {
            if !domain.contains(u, THE_PARAM_TOLERANCE) {
                continue;
            }
            u = domain.clamp(u);
        }
        let on_curve: P = value(u);
        let duplicate: bool = result
            .extrema
            .iter()
            .any(|extremum| (u - extremum.parameter).abs() < THE_PARAM_TOLERANCE);
        if duplicate || roots.roots.multiplicities[index].is_multiple_of(2) {
            continue;
        }
        let square_distance: f64 = point.square_distance_to(&on_curve);
        let is_min: bool = roots.is_minimum(index);
        if (mode == SearchMode::Min && !is_min) || (mode == SearchMode::Max && is_min) {
            continue;
        }
        result.extrema.push(PcExtremum::new(
            u,
            on_curve,
            square_distance,
            is_min,
            !is_min,
        ));
    }
    result.status = if result.extrema.is_empty() {
        Status::NoSolution
    } else {
        Status::OK
    };
}
