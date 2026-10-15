# ADR 0067: Common-carrier Bézier pair contacts

Status: implemented on a worktree branch (2026-10-15); integration is the coordinator's.

## Context

Kernel #10 needs circles and arcs in the general region pipeline. A full circle
cannot be one closed rational Bézier edge without trouble: a proper (birational)
rational parametrization maps `[0,1]` onto a proper arc only, so any single-edge
full circle is an improper (k-to-1) map. Its self-census then contains a whole
curve of parameter coincidences, and the certified pair solver correctly refuses
with an exceptional fiber. Example: the exact degree-5 circle with poles
`(-1,0),(-1,-4),(3,-2),(3,2),(-1,4),(-1,0)` and weights `5,1,1,1,1,5`.

A circle is therefore a loop of several proper arcs, as in every sketch kernel.
But two arcs of one circle share their algebraic carrier, so their pair system is
positive-dimensional (`PositiveDimensional`) or has an exceptional fiber, and the
pair census refuses every pair of such arcs. The same holds for collinear segments.

## Decision

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

## Evidence

`bezier_pair::rational_circle_tests`: exact rational circle quarters (weights
`1,1,2`) — consecutive quarters meet once at the exact tangent join with an exact
point, in both orders; opposite quarters are disjoint with a preimage at infinity;
the closing quarter meets at the seam; a reversed copy overlaps on `[0,1]`; collinear
segments meet at a shared end, are disjoint apart and refuse an overlap; arcs of
different circles keep the generic route; the quarter self domain has no contact;
version 1 still refuses. All 282 tkmath unit, 3010 integration and doc tests pass.

## Limits

Only proper conics and lines. Higher-degree arcs on one carrier (for example
degree-elevated arcs) still refuse. The pair census still requests version 1; the
general loop region proof selects version 2 explicitly.
