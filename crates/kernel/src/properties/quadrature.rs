// SPDX-License-Identifier: GPL-2.0-only
//! Adaptive Gauss-Kronrod (7/15) for trims without closed-form antiderivatives.
//! The error is the classical |K15 - G7| estimate per accepted panel: an estimate,
//! reported as such through `IntegrationMethod::AdaptiveQuadrature`.
use super::accumulate::{Moments, QUANTITIES};
use super::measured::IntegrationMethod;
use rustycad_tkernel::standard::Failure;

const KRONROD_NODES: [f64; 8] = [
    0.991_455_371_120_812_6,
    0.949_107_912_342_758_5,
    0.864_864_423_359_769_1,
    0.741_531_185_599_394_4,
    0.586_087_235_467_691_1,
    0.405_845_151_377_397_2,
    0.207_784_955_007_898_5,
    0.0,
];
const KRONROD_WEIGHTS: [f64; 8] = [
    0.022_935_322_010_529_22,
    0.063_092_092_629_978_55,
    0.104_790_010_322_250_2,
    0.140_653_259_715_525_9,
    0.169_004_726_639_267_9,
    0.190_350_578_064_785_4,
    0.204_432_940_075_298_9,
    0.209_482_141_084_727_8,
];
/// Gauss weights of the odd Kronrod nodes (indices 1, 3, 5, 7).
const GAUSS_WEIGHTS: [f64; 4] = [
    0.129_484_966_168_869_7,
    0.279_705_391_489_276_7,
    0.381_830_050_505_118_9,
    0.417_959_183_673_469_4,
];
const RELATIVE_TOLERANCE: f64 = 1e-13;
const MAX_DEPTH: u32 = 40;

/// Integral values and absolute error estimates per channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Quadrature<const N: usize> {
    pub(crate) value: [f64; N],
    pub(crate) error: [f64; N],
}

/// Integrates the `N` channels of `integrand` over `[from, to]`; Err when a panel cannot
/// reach the tolerance. Example: `integrate::<1>(&|t| Ok([t]), 0.0, 1.0)?.value[0] == 0.5`.
pub(crate) fn integrate<const N: usize>(
    integrand: &impl Fn(f64) -> Result<[f64; N], Failure>,
    from: f64,
    to: f64,
) -> Result<Quadrature<N>, Failure> {
    integrate_leading(integrand, (from, to), N)
}

/// As [`integrate`], but only the first `checked` channels decide convergence; the others
/// ride along (used for error-density channels that need no accuracy of their own).
/// Example: `integrate_leading(&|t| Ok([t, 1.0]), (0.0, 1.0), 1)?`.
pub(crate) fn integrate_leading<const N: usize>(
    integrand: &impl Fn(f64) -> Result<[f64; N], Failure>,
    interval: (f64, f64),
    checked: usize,
) -> Result<Quadrature<N>, Failure> {
    let mut sum = Quadrature {
        value: [0.0; N],
        error: [0.0; N],
    };
    let panels = Panels { integrand, checked };
    panels.refine(interval, MAX_DEPTH, &mut sum)?;
    Ok(sum)
}

/// The eight quantities of one trim as adaptive-quadrature moments; `moments(&sum)`.
pub(crate) fn moments(sum: &Quadrature<QUANTITIES>) -> Moments {
    Moments {
        value: sum.value,
        error: sum.error,
        method: IntegrationMethod::AdaptiveQuadrature,
    }
}

struct Panels<'a, F> {
    integrand: &'a F,
    checked: usize,
}

impl<const N: usize, F: Fn(f64) -> Result<[f64; N], Failure>> Panels<'_, F> {
    fn refine(
        &self,
        (a, b): (f64, f64),
        depth: u32,
        sum: &mut Quadrature<N>,
    ) -> Result<(), Failure> {
        let [kronrod, gauss, magnitude] = panel(self.integrand, a, b)?;
        // Relative to the integral of |f| (QUADPACK's resabs), so exact zeros still converge.
        let accepted = (0..self.checked)
            .all(|k| (kronrod[k] - gauss[k]).abs() <= RELATIVE_TOLERANCE * magnitude[k]);
        if accepted {
            accept(sum, &kronrod, &gauss);
            return Ok(());
        }
        if depth == 0 {
            return Err(unconverged(a, b));
        }
        let middle = 0.5 * (a + b);
        self.refine((a, middle), depth - 1, sum)?;
        self.refine((middle, b), depth - 1, sum)
    }
}

fn accept<const N: usize>(sum: &mut Quadrature<N>, kronrod: &[f64; N], gauss: &[f64; N]) {
    for k in 0..N {
        sum.value[k] += kronrod[k];
        sum.error[k] += (kronrod[k] - gauss[k]).abs() + 16.0 * f64::EPSILON * kronrod[k].abs();
    }
}

fn panel<const N: usize>(
    integrand: &impl Fn(f64) -> Result<[f64; N], Failure>,
    a: f64,
    b: f64,
) -> Result<[[f64; N]; 3], Failure> {
    let (centre, half) = (0.5 * (a + b), 0.5 * (b - a));
    let mut sums = [[0.0; N]; 3];
    for (i, node) in KRONROD_NODES.iter().enumerate() {
        add_node(&mut sums, i, half, integrand(centre - half * node)?);
        if *node != 0.0 {
            add_node(&mut sums, i, half, integrand(centre + half * node)?);
        }
    }
    Ok(sums)
}

/// Adds one sample of Kronrod node `i` to the Kronrod, Gauss and |f| sums.
fn add_node<const N: usize>(sums: &mut [[f64; N]; 3], i: usize, half: f64, value: [f64; N]) {
    let [kronrod, gauss, magnitude] = sums;
    add_weighted(kronrod, &value, KRONROD_WEIGHTS[i] * half);
    add_weighted(
        magnitude,
        &value.map(f64::abs),
        KRONROD_WEIGHTS[i] * half.abs(),
    );
    if i % 2 == 1 {
        add_weighted(gauss, &value, GAUSS_WEIGHTS[i / 2] * half);
    }
}

fn add_weighted<const N: usize>(sum: &mut [f64; N], value: &[f64; N], weight: f64) {
    for k in 0..N {
        sum[k] += weight * value[k];
    }
}

fn unconverged(a: f64, b: f64) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!(
            "adaptive Gauss-Kronrod did not reach relative tolerance {RELATIVE_TOLERANCE} on \
             [{a}, {b}] after {MAX_DEPTH} bisections; expected a smooth trim integrand"
        ),
        "",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrates_smooth_functions_within_the_estimate() {
        let sum = integrate(&|t: f64| Ok([t.exp(); QUANTITIES]), 0.0, 1.0).unwrap();
        let exact = std::f64::consts::E - 1.0;
        assert!((sum.value[0] - exact).abs() <= sum.error[0].max(1e-15));
        assert_eq!(moments(&sum).method, IntegrationMethod::AdaptiveQuadrature);
        let reversed = integrate(&|t: f64| Ok([t.sin(), 0.0]), 3.0, 0.0).unwrap();
        assert!((reversed.value[0] + (1.0 - 3f64.cos())).abs() < 1e-14);
        assert_eq!(reversed.value[1], 0.0);
        let leading = integrate_leading(&|t: f64| Ok([t, t.abs().sqrt().recip()]), (-1.0, 1.0), 1);
        assert!((leading.unwrap().value[0]).abs() < 1e-15);
    }

    #[test]
    fn refuses_integrands_it_cannot_resolve() {
        let failure = integrate(&|t: f64| Ok([t.abs().sqrt().recip()]), -1.0, 1.0);
        assert!(failure.unwrap_err().message().contains("Gauss-Kronrod"));
    }
}
