# Exact represented prism tolerance V2

## Decision

For the admitted single-Jordan polynomial prism, certify zero discrepancy in the
exact mathematical interpretation of captured binary coefficients and placement
operands. Raw entity tolerances are finite nonnegative local metadata allowances,
not measured error. Effective compatibility floors remain separately visible.
There is no inferred vertex/edge/face hierarchy and no nonzero tolerance transport.

This private evidence policy has behavior version 2. ADR0030 inspection V1 and its
incomplete placement-semantics outcome remain selectable. Frozen projection V1
and its owned-byte replay remain unchanged. This policy does not mark an ordered
validation stage passed or publish a successful body. Only the future operation
composer may do that after same-body topology, geometry-consistency and
self-intersection evidence have passed in order (RustyCADKernel issue #10).

The immutable evidence is split at that ordered boundary. One joined conformance
proof performs usable-frame, raw endpoint, upstream closure and exact trim work
and binds the complete join replay. The geometry stage consumes it before
self-intersection. The later tolerance policy accepts that already-bound proof
and applies raw metadata caps without recomputing geometry. Missing or mismatched
conformance is incomplete. Neither helper imports topology/self-intersection
certificate types or changes ordered coverage; the operation composer owns order.

## Exact placements and raw vertex transport

An actual placement is usable only when its origin and matrix are finite, its
stored scale is finite and nonzero, and its unscaled matrix determinant is
nonzero. Reuse `compare_scalar_triple_product_3d`, which evaluates the determinant
exactly. Keep scale separate: multiplying it into the matrix first can overflow,
underflow or round away invertibility. Negative scales are invertible; orientation
is a separate obligation. Existing source/chart proofs retain their support-rank
obligations. Topological metadata-owner occurrence frames and geometric
representation frames are distinct operands and both must be bound.

`BRepTool::pnt` returns a rounded transformed point. Equality to that point does
not prove exact transport from the raw vertex coordinates. ADR0035 therefore owns
additional joined per-use raw vertex rows: authored physical and directed keys,
raw local point, and actual occurrence placement, with callback-free bitwise
recheck. The original frozen world point is retained unchanged. This policy
consumes those immutable rows; it adds no capture engine or independent rebind.

The same requirement applies to the twelve directed semantic closure contexts
at Original, Census, Partition, CandidateFirst, CandidateSecond and PrismCopy.
Their joined raw operands are checked against the original captured source
query, including its existing parameter maps, exact singleton contact bounds,
interval and root options. Do not substitute a candidate curve or infer a native
endpoint from order. The shared contact-query capture supplies those operands;
the exact policy performs no live bridge query or new parameter conversion.

Reuse `PlanarResidualQuery`: the raw point is the constant plane embedding's
origin, and the actual occurrence transform is the separate plane placement.
The existing transported residual evaluator performs the exact arithmetic. The
earlier world-point adapter passes identity placement and retains its behavior.

## Complete evidence and limits

Retain six canonical endpoint obligations and all twelve directed endpoint uses,
six exact trim identities, and all eight physical raw/effective metadata rows.
Reuse a physical endpoint evaluation only after the joined directed operands
are proven identical; neither returned order nor coordinate proximity identifies
an incidence. Missing, repeated, stale or mismatched contexts prevent completion.

Every endpoint query explicitly requests zero and must return a certified squared
upper bound of zero. Preserve original incomplete/error causes and completed
work. A nonzero `Within` bound is insufficient. Exact trim evidence comes from
the existing complete source/chart identity chain, never tessellation. The
operation's requested and effective spatial/parameter limits remain unchanged;
the exact policy's zero query is recorded separately, never presented as a clamp
or a replacement operation request.

Stored caps remain explicit `NoLimit`, `Report`, or `Refuse` modes without a
default. All modes validate raw metadata. `Report` retains each exceedance;
`Refuse` prevents the policy from certifying consistency. Effective floors do not
repair raw metadata. Exact zero is within every finite nonnegative local
allowance once usable frames and exact transport are established.

## Independent fixtures

- Singular frames can hide different local points and must be refused.
- An exact determinant of one survives cancellation of rounded products at
  `2^52`; finite extreme nonzero scales remain separate from matrix arithmetic.
- Local point `[1,0,0]` translated by `[2^53,0,0]` rounds to `[2^53,0,0]` through
  `BRepTool::pnt`. A constant curve there passes the earlier world-point query but
  fails exact raw transport, whose discrepancy is one.
- Raw zero/subfloor/invalid values and `1/8` against cap `1/16` retain independent
  metadata outcomes while exact geometry stays unchanged.
- Noncommuting placements, reversal, complete seam contexts, missing/stale rows,
  typed cancellation and independent process replay remain explicit gates.

## Migration boundary

`EXACT-PRISM-TOLERANCE-V2-COMPOSE` is due at the Martensite C1 milestone gate.
Integrate this evidence into one complete ordered operation with same-body proof
joining, preserve its diagnostics/history/requested settings, and validate the
declared analytic corpus before recording replay migration and deleting
superseded temporary composition. This decision does not complete general
profiles, public API contracts, document naming or application acceptance.
