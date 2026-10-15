# ADR 0068: General loop region proof for analytic profiles

Status: implemented on a worktree branch (2026-10-15); integration is the coordinator's.

## Context

The certified region proofs stopped at one closed polynomial edge (single boundary)
and two nested closed polynomial edges (ADR 0046/0058). Sketch profiles have loops of
several edges (lines, arcs, splines), circles are loops of proper rational arcs
(ADR 0067), and a profile may have many nested and side-by-side loops with several
selected regions. ADR 0014 requires one general pipeline, not a circle path.

## Decision

### Owned profile preparation

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
(`enumerate_carrier_census`, ADR 0067), partition, candidate copies and contact
transport.

### Loop region proof

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
5. Area: `certify_rational_face_area` (tkmath, ADR 0066) encloses each loop's Green
   integral, requires it to exclude zero (its sign is the loop orientation and its
   magnitude the enclosed area) and returns `|outer| - sum |hole|` with the unchanged
   certified error cap.

Cancellation is polled between stages and inside every numerical stage; the observed
caller stop classifies the refusal. The certificate records loops with parents and
witnesses, every join, every region with its outer loop, holes, per-loop seed
windings and certified area, and the number of pair domains and of carrier pairs.

## Evidence

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

## Limits

The proof publishes no body. Region-to-candidate binding (which partition face is
which selected region), prism construction, ordered validation, history, measures
beyond chart area and naming for general profiles are open. Loops that touch or
cross, edges shared by two loops (adjacent selected faces sharing an edge), tangent
contacts between different carriers, improper or degree-elevated arcs on one carrier
and non-default planes beyond the census chart are refused or untested. Areas are in
the census chart; world area needs the plane Gram factor (ADR 0044 style).
