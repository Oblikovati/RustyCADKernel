# Next bounded tolerance-consistency composition

This is a scoped follow-on proposal approved by the integration owner, not an
implemented certificate or a claim that Kernel issue #10 is complete. Finish and
integrate ADR0030 evidence first, delete its worktree, then start from current
`main`. Record a new kernel decision and explicit behavior version. Keep private
inspection V1 and the earlier incomplete receipt path available for replay.

## Declared meanings

- Represented geometry is the exact mathematical interpretation of the captured
  binary coefficients, parameter ranges and placement operands. Exact identities
  and certified zero residuals concern that representation, not manufacturing
  accuracy or uncertainty in an external source.
- Raw entity tolerance is a local stored metadata allowance. It must be finite
  and nonnegative. It is not a measured error. Do not infer vertex >= edge >= face.
- The compatibility accessor's floor is retained separately. It neither repairs
  invalid raw metadata nor provides acceptance evidence.
- The operation's requested world-spatial limit remains unchanged. A separate,
  explicitly reported exact-consistency query requests zero, and must prove zero.
  A nonzero requested-bound result is insufficient even when it is `Within`.
- Raw metadata caps retain explicit `NoLimit`, `Report`, and `Refuse` semantics.
  `Report` preserves every exceedance as a diagnostic. `Refuse` prevents the
  tolerance stage from passing. No mode bypasses finite/nonnegative validation.

## Required composition obligations

1. Declare the admitted operation/body profile. Consume successful topology,
   geometry-consistency and self-intersection certificates in that order before
   allowing the tolerance stage to pass. All must describe the same operation,
   source revisions, options, actual body and owned geometry; missing or stale
   evidence remains incomplete.
2. Reuse the reviewed current-binding path and endpoint census. Require complete
   source/copy/history binding, all actual directed contexts, a bijection to the
   six canonical physical endpoint obligations and all twelve directed uses,
   and all six directed trims. Do not infer identity from rank or coordinates.
3. Require every endpoint's existing exact scalar query to return a zero squared
   upper bound and every complete trim identity chain to prove zero. Preserve
   endpoint parameter, point, curve/placement operands and actual directed
   contexts. `Exceeds`, invalid/unsupported operands, cancellation or incomplete
   proof prevents tolerance completion. No alternate evaluator or retry path.
4. Bind all raw/effective metadata rows to those same actual entities. Validate
   raw values and apply the requested raw cap without changing values. Exact zero
   discrepancy is within every nonnegative local allowance; this limited policy
   does not need to approximate a world uncertainty radius from a stored scale.
5. Validate the captured placement conditions used by the prior geometry and
   embedding certificates. Do not extrapolate the zero argument to nonzero
   discrepancies or use a rounded scale factor as an operator-norm proof.
   Nonzero transport requires a later explicit inverse-metric/transport policy.
6. After all external callbacks, replay existing source/chart binding and local
   census, endpoint and metadata operands without callbacks. Compose into the
   immutable publication through actual identity bindings and stable keys,
   retaining each proof's provenance; do not manufacture a graph-wide statement
   from one successful append or a scalar-only replay summary.
7. Return completion, diagnostics, ordered coverage, composed history and
   achieved error together. A certified aggregate zero is available only after
   all required geometric incidence proofs are complete. Keep raw metadata,
   effective compatibility values, requested limits, query settings and achieved
   certified residuals separate in the result.

## Independent acceptance fixtures

- Current authored closed cubic prism: six canonical endpoint bounds and six trim
  identities exactly zero, complete directed incidence, all preceding ordered
  stages complete, and one valid named solid through the intended consumer.
- Raw zero and subfloor values remain visible despite identical accessor floors.
  Raw NaN, negative and infinite metadata never pass, including explicit NoLimit.
- Raw `1/8` versus cap `1/16`: Report retains the exceedance; Refuse blocks the
  stage. Geometric residual stays zero and no geometry/metadata is healed.
- A requested world budget of `1/8` never admits a nonzero discrepancy under this
  exact policy. Preserve the earlier PrismCopy closure refusal for an aliased
  retained-vertex mutation. Use owned operands to independently pin threshold
  arithmetic without claiming body admission for changed snapshots.
- Noncommuting placement, uniform scaling, reversal and both seam uses preserve
  exact-zero evidence and original metadata bits. Wrong transform order fails.
- Missing/duplicated incidence, stale point/range/curve/metadata, mismatched proof
  provenance, source mutation, cancellation and exhausted upstream proof work
  remain incomplete with original typed causes and affected entities.
- Compare complete immutable result replay across processes and worker counts;
  the current arithmetic-only receipt is insufficient for this publication gate.

## Migration boundary

Compose this policy within Martensite C1 / issue #13 and the Kernel issue #10
capability work. Keep release acceptance separate: general declared profiles,
public contracts, document naming/history behavior and application evidence are
still required. Delete superseded temporary composition paths only after the
recorded corpus and complete-operation gate pass within that milestone.
