// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Gauss.cxx (the
// adaptive Compute overloads and the file-static interval helpers) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The adaptive `Compute` of `BRepGProp_Gauss`: the outer integral along the boundary
//! curves (or the V range of a naturally bounded face) with interval bisection driven
//! by the difference of two Gauss orders; the inner U integral is in
//! `gauss_adaptive_u.rs`.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::gauss::{BRepGPropGauss, GaussProps, GaussType, Inertia};
use super::gauss_table::{GaussRule, gauss_points_and_weights};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math;

/// The minimal range of an interval (`EPS_PARAM`).
pub(super) const EPS_PARAM: f64 = 1.0e-12;
/// The ratio of the orders of the error estimate (`ERROR_ALGEBR_RATIO`).
const ERROR_ALGEBR_RATIO: f64 = 2.0 / 3.0;
/// The bisection depth per knot interval (`SUBS_POWER`).
pub(super) const SUBS_POWER: usize = 32;

/// `SM`: the native size of the interval arrays, `SUBS_POWER * GaussPointsMax + 1`.
pub(super) fn sm() -> usize {
    SUBS_POWER * math::gauss_points_max() as usize + 1
}

/// `maxSubs(theN, theCoeff)`: `theN * theCoeff + 1`, IntegerLast when it overflows.
pub(super) fn max_subs(n: usize, coeff: usize) -> usize {
    let last = i32::MAX as usize;
    if last / coeff < n {
        last
    } else {
        n * coeff + 1
    }
}

/// The two Gauss rules (orders `n` and `ceil(2 n / 3)`) of one direction.
pub(super) struct GaussRules {
    pub(super) orders: [i32; 2],
    pub(super) rules: [GaussRule; 2],
}

impl GaussRules {
    pub(super) fn new(order: i32) -> Result<Self, Failure> {
        let second = real_to_int((ERROR_ALGEBR_RATIO * f64::from(order)).ceil());
        Ok(Self {
            orders: [order, second],
            rules: [
                gauss_points_and_weights(order)?,
                gauss_points_and_weights(second)?,
            ],
        })
    }
}

/// The interval arrays of one direction (`anInertiaL`, `L1`, `L2`, `ErrL` or their U
/// twins), of the native size SM, grown by `fillIntervalBounds`.
pub(super) struct Intervals {
    pub(super) inertia: Vec<Inertia>,
    pub(super) first: Vec<f64>,
    pub(super) second: Vec<f64>,
    pub(super) error: Vec<f64>,
}

impl Intervals {
    fn new() -> Self {
        Self {
            inertia: vec![Inertia::default(); sm()],
            first: vec![0.0; sm()],
            second: vec![0.0; sm()],
            error: vec![0.0; sm()],
        }
    }

    /// `fillIntervalBounds(theA, theB, theKnots, theNumSubs, ...)`: grows the arrays
    /// (the error arrays zeroed) when the knots need more room, then writes the
    /// intervals of `[a, b]` cut at the knots strictly inside (searched with libstdc++'s
    /// `upper_bound` and `lower_bound`, whatever the knot order); their count.
    pub(super) fn fill(
        &mut self,
        a: f64,
        b: f64,
        knots: &[f64],
        common: Option<&mut Vec<f64>>,
    ) -> usize {
        let size = knots
            .len()
            .max(max_subs(knots.len().wrapping_sub(1), SUBS_POWER));
        if size > self.first.len() {
            self.inertia.resize(size, Inertia::default());
            self.first.resize(size, 0.0);
            self.second.resize(size, 0.0);
            self.error = vec![0.0; size];
            if let Some(common) = common {
                *common = vec![0.0; size];
            }
        }
        let first_knot = upper_bound(knots, 0, a);
        let last_knot = lower_bound(knots, first_knot, b);
        let count = last_knot - first_knot;
        self.first[0] = a;
        self.first[1..=count].copy_from_slice(&knots[first_knot..last_knot]);
        self.second[..count].copy_from_slice(&knots[first_knot..last_knot]);
        self.second[count] = b;
        count + 1
    }

    /// The `Mass` of every interval set to 0 and the errors zeroed.
    pub(super) fn reset(&mut self) {
        self.inertia
            .iter_mut()
            .for_each(|inertia| inertia.mass = 0.0);
        self.error.iter_mut().for_each(|error| *error = 0.0);
    }

    /// The bisection step of the loop: the next interval `j` (1-based count) is the
    /// first one while `j` does not pass `end`, else the upper half of the interval of
    /// largest error; the pair of intervals to compute.
    pub(super) fn next_range(&mut self, j: usize, end: usize) -> [usize; 2] {
        if j <= end {
            return [j - 1, 0];
        }
        let worst = max_element(&self.error);
        self.first[j - 1] = (self.first[worst] + self.second[worst]) * 0.5;
        self.second[j - 1] = self.second[worst];
        self.second[worst] = self.first[j - 1];
        [worst, j - 1]
    }

    /// `JL == LMaxSubs || |L2(JL - 1) - L1(JL - 1)| < EPS_PARAM`: the interval cannot be
    /// computed.
    pub(super) fn is_exhausted(&self, j: usize, max: usize) -> bool {
        j == max || (self.second[j - 1] - self.first[j - 1]).abs() < EPS_PARAM
    }
}

/// libstdc++ `std::upper_bound(first, last, value)` over `values[from..]`.
fn upper_bound(values: &[f64], from: usize, value: f64) -> usize {
    let (mut first, mut len) = (from, values.len() - from);
    while len > 0 {
        let half = len >> 1;
        if value < values[first + half] {
            len = half;
        } else {
            first += half + 1;
            len -= half + 1;
        }
    }
    first
}

/// libstdc++ `std::lower_bound(first, last, value)` over `values[from..]`.
fn lower_bound(values: &[f64], from: usize, value: f64) -> usize {
    let (mut first, mut len) = (from, values.len() - from);
    while len > 0 {
        let half = len >> 1;
        if values[first + half] < value {
            first += half + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    first
}

/// `std::max_element(begin, end) - begin`: the first largest value.
pub(super) fn max_element(values: &[f64]) -> usize {
    let mut largest = 0;
    for (index, value) in values.iter().enumerate().skip(1) {
        if values[largest] < *value {
            largest = index;
        }
    }
    largest
}

/// The inputs of one adaptive computation.
pub(super) struct AdaptiveInput<'a> {
    pub(super) location: Point3,
    pub(super) coeff: &'a [f64],
    pub(super) by_point: bool,
    pub(super) epsilon: f64,
    pub(super) error_passes: usize,
    pub(super) bounds: (f64, f64, f64, f64),
    pub(super) natural: bool,
    pub(super) u_rules: GaussRules,
    pub(super) u_knots: Vec<f64>,
}

/// The variables the native Compute keeps across the boundary curves and loops.
#[derive(Default)]
pub(super) struct AdaptiveState {
    pub(super) eps: f64,
    pub(super) eps_l: f64,
    pub(super) eps_u: f64,
    pub(super) error_u: f64,
    pub(super) error_l_max: f64,
    pub(super) dul: f64,
    pub(super) v: f64,
    pub(super) u2: f64,
}

impl BRepGPropGauss {
    /// `Compute(theSurface, theDomain, theLocation, theEps, theCoeff, theIsByPoint, ...)`:
    /// adaptive integration of the integrator's type (no type check, as natively). A
    /// negative `eps` or one below 0.001 estimates the error with a second Gauss order;
    /// one in `(0, 0.001)` also refines the intervals until the error is below it. A
    /// face without wires is integrated on its natural bounds, else along each edge of
    /// the domain: an edge without pcurve returns Precision::Infinite() and leaves `out`
    /// as it was. Returns the reached relative error (`|eps|` without estimate);
    /// `gauss.compute_adaptive(&mut face, &mut domain, location, 1.0e-6, &[0.0; 3], true, &mut out)?`.
    #[allow(clippy::too_many_arguments)] // The native overload's arguments.
    pub fn compute_adaptive(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        eps: f64,
        coeff: &[f64],
        by_point: bool,
        out: &mut GaussProps,
    ) -> Result<f64, Failure> {
        let is_error_calculation = 0.0 > eps || eps < 0.001;
        let is_verify = 0.0 < eps && eps < 0.001;
        let epsilon = eps.abs();
        let bounds = surface.bounds()?;
        self.check_bounds(bounds);
        let natural = surface.get_face().nb_children() == 0;
        let u_rules = GaussRules::new(surface.s_int_order(epsilon)?)?;
        let u_knots = knots_of(surface.su_int_subs()?, |k| surface.u_knots(k))?;
        let input = AdaptiveInput {
            location,
            coeff,
            by_point,
            epsilon,
            error_passes: if is_error_calculation { 2 } else { 1 },
            bounds,
            natural,
            u_rules,
            u_knots,
        };
        let mut run = AdaptiveRun::new(self, input, is_verify);
        let Some(total) = run.boundaries(surface, domain)? else {
            return Ok(precision::infinite());
        };
        self.convert(&total, (coeff, by_point), out);
        Ok(run.reached_error(out.mass))
    }

    /// `Compute(theSurface, theDomain, theLocation, theEps, Mass, G, I)`: the adaptive
    /// surface integration (Sinert only, Standard_ProgramError otherwise);
    /// `gauss.compute_adaptive_surface(&mut face, &mut domain, location, 1.0e-6, &mut out)?`.
    pub fn compute_adaptive_surface(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
        location: Point3,
        eps: f64,
        out: &mut GaussProps,
    ) -> Result<f64, Failure> {
        self.assert_type(GaussType::Sinert)?;
        self.compute_adaptive(surface, domain, location, eps, &[], true, out)
    }
}

/// A knot array of `subs + 1` values filled by one of the knot members of the face.
pub(super) fn knots_of(
    subs: i32,
    fill: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
) -> Result<Vec<f64>, Failure> {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, subs + 1)
        .map_err(|error| error.into_failure("BRepGProp_Gauss::Compute"))?;
    fill(&mut knots)?;
    Ok(knots.iter().copied().collect())
}

/// One adaptive computation: the integrator, its inputs, its loop state and the
/// interval arrays of both directions.
pub(super) struct AdaptiveRun<'g, 'a> {
    pub(super) gauss: &'g BRepGPropGauss,
    pub(super) input: AdaptiveInput<'a>,
    pub(super) is_verify: bool,
    pub(super) state: AdaptiveState,
    pub(super) l: Intervals,
    pub(super) u: Intervals,
    pub(super) err_ul: Vec<f64>,
}

impl<'g, 'a> AdaptiveRun<'g, 'a> {
    fn new(gauss: &'g BRepGPropGauss, input: AdaptiveInput<'a>, is_verify: bool) -> Self {
        Self {
            gauss,
            input,
            is_verify,
            state: AdaptiveState::default(),
            l: Intervals::new(),
            u: Intervals::new(),
            err_ul: vec![0.0; sm()],
        }
    }

    /// `ErrorLMax / |Mass|` (0 for a null mass) with the error estimate, `|eps|` else.
    fn reached_error(&self, mass: f64) -> f64 {
        if self.input.error_passes != 2 {
            return self.input.epsilon;
        }
        if mass != 0.0 {
            self.state.error_l_max / mass.abs()
        } else {
            0.0
        }
    }
}
