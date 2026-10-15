# ADR 0066: Certified analytic profile regions

Status: implemented on a worktree branch (2026-10-15); integration is the coordinator's.

Kernel #10 needs circles, arcs, lines and splines in the general region pipeline
(ADR 0014). This ADR has three parts that land together: certified rational
boundary and sweep integrals (part A), common-carrier Bézier pair contacts (part B)
and the general loop region proof for analytic profiles (part C). Code comments
cite them as "ADR 0066 part A", "part B" and "part C".

## Part A: certified rational boundary and sweep integrals

### Context

The certified Green and sweep integrals of `metric_bounds::polynomial_integrals`
refuse every rational weight, because for a non-constant denominator the exact
value is in general irrational (a circle gives `pi`). AGENTS.md requires area and
volume from error-bounded integration.

### Decision

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

### Evidence

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

### Limits

This is a scalar. Region area and solid volume still require the separate coverage,
orientation and injectivity proofs of the region pipeline (part C). The older tkbo
measure stages still call the polynomial entries.

## Part B: common-carrier Bézier pair contacts

### Context

A full circle cannot be one closed rational Bézier edge without trouble: a proper
(birational) rational parametrization maps `[0,1]` onto a proper arc only, so any
single-edge full circle is an improper (k-to-1) map. Its self-census then contains a
whole curve of parameter coincidences, and the certified pair solver correctly
refuses with an exceptional fiber. Example: the exact degree-5 circle with poles
`(-1,0),(-1,-4),(3,-2),(3,2),(-1,4),(-1,0)` and weights `5,1,1,1,1,5`.

A circle is therefore a loop of several proper arcs, as in every sketch kernel.
But two arcs of one circle share their algebraic carrier, so their pair system is
positive-dimensional (`PositiveDimensional`) or has an exceptional fiber, and the
pair census refuses every pair of such arcs. The same holds for collinear segments.

### Decision

`BezierPairPolicy.version = 2` adds an exact common-carrier route for distinct
pairs; version 1 is unchanged (its refusal is pinned by a test). Self domains keep
the generic route.

- Classify each source from its exact homogeneous power coefficients
  `Q(t) = sum_k V_k t^k`, `V_k` in `Z^3`. Degree two with `det[V0 V1 V2] != 0` is a
  proper conic parametrization (Veronese map composed with an invertible linear
  map); degree one with `V0 x V1 != 0` is a proper line. Anything else keeps the
  generic route.
- The second source shares the first's carrier when its whole power form satisfies
  the first carrier's implicit equation identically: `n . V_k = 0` for a line with
  `n = V0 x V1`; `L1(u)^2 = L0(u) L2(u)` for a conic, with `L_i = adj_i . Q(u)` and
  `adj` the adjugate rows (`adj X` is proportional to `(1, t, t^2)`).
- Each carrier point has one projective preimage (`t = adj_1.X / adj_0.X`, or
  `t = n.(V0 x X) / n.(X x V1)`, infinity on a zero denominator). The second
  interval maps to the projective arc from its start preimage to its end preimage
  through its midpoint preimage. Its intersection with the first interval is exact
  interval logic over rationals, including arcs through infinity.
- A single meeting point is a contact at exact source endpoints `(t, u)`; its world
  point is the existing exact envelope of the first curve. A positive-length meet
  is a typed `CarrierOverlap` refusal with its outward parameter range; reversed or
  collapsed intervals are `CarrierDegenerate`. Both denominators are certified
  root-free on their intervals as before.

The certificate exposes `common_carrier()` (kind and the three exact preimages) and
`sign_stage() == SkippedCommonCarrier`; `graph()` and `constant_exclusion()` stay
`None`, so consumers that require one of those continue to refuse.

### Evidence

`bezier_pair::rational_circle_tests`: exact rational circle quarters (weights
`1,1,2`): consecutive quarters meet once at the exact tangent join with an exact
point, in both orders; opposite quarters are disjoint with a preimage at infinity;
the closing quarter meets at the seam; a reversed copy overlaps on `[0,1]`; collinear
segments meet at a shared end, are disjoint apart and refuse an overlap; arcs of
different circles keep the generic route; the quarter self domain has no contact;
version 1 still refuses. All 282 tkmath unit, 3010 integration and doc tests pass.

### Limits

Only proper conics and lines. Higher-degree arcs on one carrier (for example
degree-elevated arcs) still refuse. The pair census still requests version 1; the
general loop region proof (part C) selects version 2 explicitly.

## Part C: general loop region proof for analytic profiles

### Context

The certified region proofs stopped at one closed polynomial edge (single boundary)
and two nested closed polynomial edges (ADR 0046/0058). Sketch profiles have loops of
several edges (lines, arcs, splines), circles are loops of proper rational arcs
(part B), and a profile may have many nested and side-by-side loops with several
selected regions. ADR 0014 requires one general pipeline, not a circle path.

### Decision

#### Owned profile preparation

`owned_request::profile::prepare_owned_profile` takes owned edges (polynomial or
positive-weight rational Bézier poles, caller source id and revision, caller start
and end vertex ids), authored loops (cyclic lists of directed edge uses), selected
regions (name and chart seed), a declared plane and explicit child policies.

Admission refuses, with typed operands and before any construction: unsupported
version, empty profile or selection, nonfinite plane or seed, bad pole count,
nonfinite pole, weight count, nonpositive or nonfinite weight, zero or duplicate
edge/loop/region/operation names, disconnected loops (end vertex id differs from the
next start id), unknown or reused edges, unused edges, and equal vertex ids with
bitwise-different endpoint poles. Construction creates one native vertex per caller
vertex id (shared ids share the vertex object; equal coordinates never merge) and one
Bézier edge per owned edge. The existing general stages then run unchanged:
boundary capture, the source-bound census with pair policy version 2
(`enumerate_carrier_census`, part B), partition, candidate copies and contact
transport.

#### Loop region proof

`region_certificate::loop_region::inspect_loop_regions` consumes the shared
`PairedContactTransaction` and the authored hypotheses. It solves no new
intersection query.

1. Binding: every source is exactly one unsplit, forward, full native `[0,1]` census
   segment with exactly its own original parent; every source is used exactly once.
2. Joins: the census holds all `n(n+1)/2` pair and self domains, each with a complete
   current certificate (the first incomplete pair is named), and no exhaustive-pair or
   failed-work obligation. Every certified contact must be an authored join at exact
   native endpoints (`t, u` in `{0,1}`, checked by exact dyadic comparison): the end of
   one use with the start of the next, or the closure `t=0,u=1` of a one-edge loop.
   Every authored join must be certified. Anything else refuses with the contact's
   world point. Continuous injective edges meeting only at consecutive joins make each
   loop a Jordan curve and the loops pairwise disjoint.
3. Nesting: for disjoint Jordan loops a point of one loop decides containment. The
   exact start point of each loop's first use is located against every other loop by
   the certified multi-occurrence wire winding (`classify_bezier_wire`, rational
   weights admitted). Windings must be `0` or `+-1`; containers must form a strict
   chain; the innermost container is the parent.
4. Regions: each seed's exact windings against all loops (a seed on a loop refuses).
   The containing loops must form a chain; the innermost is the face's outer loop; its
   children not containing the seed are the holes. No containing loop is an unbounded
   selection; two seeds in one face are a duplicate selection.
5. Area: `certify_rational_face_area` (tkmath, part A) encloses each loop's Green
   integral, requires it to exclude zero (its sign is the loop orientation and its
   magnitude the enclosed area) and returns `|outer| - sum |hole|` with the unchanged
   certified error cap.

Cancellation is polled between stages and inside every numerical stage; the observed
caller stop classifies the refusal. The certificate records loops with parents and
witnesses, every join, every region with its outer loop, holes, per-loop seed
windings and certified area, and the number of pair domains and of carrier pairs.

### Evidence

`owned_request::profile` tests (17): a rational circle of four exact quarters (area
`pi`, ten domains, four tangent joins, six carrier pairs), a clockwise circle
(winding `-1`), circle-in-circle annulus and inner disc (`3.75 pi`, `0.25 pi`), a square
with a circular hole (`16 - pi`), side-by-side circles, three nested loops with three
selected faces (`16 pi - 16`, `16 - pi`, disc), a slot of two lines and four tangent
arcs (`4 + pi`), the closed polynomial cubic through its closure contact (`2.4`), and a
two-edge lens; replay is identical and inputs are untouched. Refusals: weights,
disconnected loop, shared-vertex mismatch, empty selection, crossing circles
(unexpected contact), externally tangent circles (named incomplete pair), seed
outside all loops, seed on a loop, duplicate selection, unsupported version, the
improper one-edge degree-5 circle (its self pair is incomplete), and cancellation
at every early checkpoint and a deterministic stride through the numerical stages.
`rational_integrals` face tests: annulus area for all four orientation combinations,
square minus two circular holes, and a zero-area loop's indeterminate orientation.

### Limits

The proof publishes no body. Region-to-candidate binding (which partition face is
which selected region), prism construction, ordered validation, history, measures
beyond chart area and naming for general profiles are open. Loops that touch or
cross, edges shared by two loops (adjacent selected faces sharing an edge), tangent
contacts between different carriers, improper or degree-elevated arcs on one carrier
and non-default planes beyond the census chart are refused or untested. Areas are in
the census chart; world area needs the plane Gram factor (ADR 0044 style).
