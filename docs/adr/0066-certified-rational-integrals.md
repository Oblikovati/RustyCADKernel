# ADR 0066: Certified rational boundary and sweep integrals

Status: implemented on a worktree branch (2026-10-15); integration is the coordinator's.

## Context

Kernel #10 needs circles, arcs and other positive-weight rational curves in the
general region pipeline (ADR 0014). The certified Green and sweep integrals of
`metric_bounds::polynomial_integrals` refuse every rational weight, because for a
non-constant denominator the exact value is in general irrational (a circle gives
`pi`). AGENTS.md requires area and volume from error-bounded integration.

## Decision

Add `certify_rational_boundary_integral` and `certify_rational_sweep_integral`
beside the polynomial entries. They take the same `OrientedMetricCurve` slice,
the same native `[0,1]` range, the same homogeneous coefficients
(`coefficients::embedded`) and the same Green numerator `P = X Y' - Y X'`. The
integrand of one curve is `P / (2 W^2)`; the `W'` terms cancel exactly.

- Constant denominator (polynomial curves, lines, equal weights): the existing
  exact rational integration; evidence `ExactRational`, and the report equals the
  polynomial report bit for bit.
- Otherwise: adaptive bisection of dyadic spans `[j, j+1] / 2^l`. On each span the
  substitution `t = (2j + 1 + v) / 2^(l+1)` gives integer polynomials `p`, `q`. A
  binary64 power series of order 16 (`RATIONAL_TAYLOR_ORDER`, fixed for version 1)
  only *chooses* a dyadic approximant `S`; the remainder `R = p - S q` is computed
  exactly. `S` integrates exactly over `[-1,1]`. With `v = 2u - 1`, `R/q` is a convex
  combination of the scaled Bernstein ratios `r_i / q_i` (the common binomial factor
  cancels), so its integral lies in `2 [min, max]`. Positive weights make every
  Bernstein coefficient of `q = W^2` positive on every span; a nonpositive one is a
  typed `DenominatorSign` refusal, never a silent fallback.
- A span is accepted when its exact width is at most its proportional share of
  `cap / (curves * |sweep determinant|)`. The exact bounds are summed, reversed
  occurrences swap and negate, the exact determinant multiplies (swapping for a
  negative sign), and only the final endpoints are rounded outward. The
  representative is the lower rounding of the exact midpoint; the published error
  bound is the upward rounding of the exact distance to the farther endpoint and is
  refused when above the unchanged cap.

Options add `max_subintervals` per curve (1..=65536, default 4096). Unmet shares are
a typed `WorkLimit` with the certified parameter length; nothing partial is
published. Cancellation is polled in every polynomial step. There is no quadrature
estimate, sampled sign, tolerance widening or retry.

## Evidence

`rational_integrals` tests (22): exact rational unit-circle quarters
(`x = (1-t^2)/(1+t^2)`, weights `1,1,2`) give `pi/4`, full circles `pi r^2` at three
centers and radii and both traversals, an axis-scaled ellipse `pi a b`, a half disc
closed by a line `pi/2`, sweeps `pi r^2 h` for three heights of both signs; each
40-digit rational enclosure of the closed form lies inside `representative +- error`
and the error meets the cap. Mobius reparametrizations (weights times `lambda^i`,
up to `lambda = 64`) and common weight scaling give the same value; an arbitrary
rational cubic loop and a degree-16 loop are invariant under exact translation,
rotation and scaling. Polynomial and equal-weight curves match the polynomial entry
exactly. Point and collinear rational curves give exact zero even with a zero cap;
a singular sweep gives exact zero. Policy, count, degree, weight, work and error-cap
refusals are typed; every cancellation checkpoint refuses with clean reuse; replay
is identical across calls and threads, and inputs are untouched. Looser caps never
need more work.

## Limits

This is a scalar. Region area and solid volume still require the separate coverage,
orientation and injectivity proofs of the region pipeline. The tkbo measure stages
still call the polynomial entries; routing rational boundaries through them is part
of the rational region proof.
