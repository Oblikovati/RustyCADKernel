# Joined prism orientation, behavior version 1

## Decision

Kernel#10 ordered validation needs the existing outward/trim orientation proof
(ADR0034) over the exact same immutable body as topology, geometry and tolerance.
Add a private pure joined orientation operation consuming ADR0035's
`JoinedPrismProof`. It performs no live capture, source reclassification, topology
traversal or endpoint evaluation. It returns no valid body.

First extract ADR0034's existing borrowed sign/conclusion evaluation without
changing the live entry's capture order, error causes, options, callback positions
or final callback-free rebind. The original portable JSON and the 1,949-checkpoint
cancellation receipt were pinned before extraction. Existing live behavior stays
selectable. This additive private operation introduces no public API break and
replaces no production engine; no deletion migration is due.

## Interface and operands

`inspect_joined_prism_orientation(join, JoinedPrismOrientationOptions::v1(), cancel)`
returns an immutable owned certificate or typed failure. The only new option is
an explicit behavior version. Embedding/regularity options come unchanged from
`join.embedding().requested()`; this operation introduces no metric tolerance,
normalization, integral or approximation budget.

The borrowed evaluator reuses exact Jacobian sign, explicit
`DerivedFromFreshFacedWinding` provenance and all existing face/trim equations.
Thus `q=f*r*w` still distinguishes increasing stored curve parameter, actual
boundary traversal and selected faced winding. The stored side parameter
`v=-h*s` still reverses the side parameter normal. No signed-volume substitution
or additional winding engine is used.

The join already authenticates each actual theorem chart/trim against its
caller-authored face/use/source names and full original-parent multiset. Roles
select these existing mappings with exact cardinality checks, never invent names
from returned order or coordinates. Every mapped face and trim is compared with
its actual cumulative `FrozenPrismUse.orientation`. Exact trim interval bits and
parent multiplicities must agree with the borrowed theorem. Missing, duplicate
or inconsistent named operands prevent a successful certificate.

The certificate retains all three named face and six trim conclusions, including
both authored seam occurrences, all named parent vectors and full output parent
disposition vectors without deduplication. Original-name mapping and history
classification remain responsibilities of the join. This operation does not
reconstruct them from raw addresses or collapse a parent multiset to a set.

## Binding, ownership and ordered composition

The certificate owns complete `join.replay_bytes()`, including the frozen
projection, theorem, all exact source/chart/trim operands, authored contexts,
parent multiplicities, options and closure/raw-point sidecars. Binding uses exact
byte equality, never a hash or pointer identity. Named conclusions are serialized
in total authored-use order; all numerical operand bits remain in the complete
join binding. The certificate contains no mutable geometry handle or live cache.

The ordered validator must use this after topology, together with geometry and
endpoint/tolerance evidence over the same joined projection. A successful
orientation certificate establishes only conditional exact directions. Geometry
consistency, allowed incidence, complete history, tolerance policy, public result
contracts and Kernel#10 corpus acceptance remain their respective gates.

Mutating an original live shape after a join was captured does not change this
owned mathematical input or its certificate. Live freshness is the join owner's
explicit final rebind responsibility. A different owned join cannot borrow a
certificate unless its complete replay bytes agree. This pure operation must not
silently perform another mutable inspection to establish that relationship.

## Completion and errors

The operation polls once after version admission and once after bounded sign/name
work, before returning a certificate. It preserves no partial success on
cancellation. Unsupported version, prerequisite direction refusal, named table
cardinality, missing use, actual direction mismatch, interval-bit mismatch and
parent-multiplicity mismatch have distinct typed causes. Failures retain operation,
requested version, phase, named context when available and the unchanged nested
legacy cause. The ordered consumer classifies these causes explicitly.

## Verification

Preserve the old replay and exact cancellation causes before/after extraction.
Use independently expected cubic pole/traversal/extrusion orientation fixtures,
all three faces and six trims, role/name permutations, missing/duplicate mappings,
full join mismatch, parent multiplicity and exact interval distinctions. Keep
isolated malformed table tests separate from joins admitted by the production
constructor. Exercise immutable source isolation, cancellation/reuse, Send/Sync,
separate process and concurrent replay. Record source bookends, all affected
checks and sanitizer limitations. No native OCCT oracle, tessellation acceptance,
retry ladder, tolerance widening or body promotion is permitted.
