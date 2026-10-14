# ADR0032: bounded observed prism copy lineage

Status: bounded private implementation; behavior version 1.
Basis: `78662a49273a704d585b38e673f2b900a2e3b342`.

## Decision and Interface

Preserve the exact `BRepBuilderAPICopy::modified` member query, native image,
orientation-adjusted image, and all original relation classes at the existing
copier call. These operation-local observations retain Shape aliases. They are
historical provenance, never proof of immutable ownership or current geometry.
The structural retention commit leaves the existing flattened relations,
classification, gaps, body output, and frozen projection replay unchanged.

`frozen::lineage::inspect_prism_lineage` combines those observations with existing
sweep bindings and the existing frozen diagnostic projection. Its outcomes are
Inspected, Incomplete, Failed, and Cancelled. Inspected means that this bounded
set of observed copy/sweep operands has been checked and captured. It never
means a successful body, completed upstream history, or ordered validation.
Member and directed correspondence coverage are separate. Typed findings retain
operation, stage, known authored entity, actual/expected counts or original
parent classes, and the underlying owned cause. Frozen failures retain their
original operation, phase, entity and cause rather than only their message.

## Identity and ownership

Original and output keys reuse ADR0023 bindings and naming. Additive authored
bindings name selected members and selected/copied directed contexts. A copied
member is named by the existing authored source binding. Intermediate keys use
operation, Selected/Copied stage, authored member and kind. Directed keys contain
the full authored ancestry. Exact operation-local identities locate bindings;
addresses, coordinates, parent sets, hashes and returned positions never become
stable keys. Two same-kind intermediates with equal geometry and original parent
sets remain distinct when explicitly authored as distinct.

A native copy result establishes only a member observation. Directed pairing
requires the actual selected and copied contexts, a unique mapped member at
every ancestor, and equal placement and orientation. Caller-provided use labels
alone cannot establish correspondence. Indistinguishable repeated uses remain
typed incomplete even if the caller supplies distinct labels. Actual child order
is retained as an operand; it is not the identity or pairing rule.

The root inspection owns standard collections, exact placement bits, scalar
keys, parent classes, typed findings, and an optional owned frozen projection.
Borrowed immutable views expose no mutable accessor. No Shape, Handle, Report,
cache, graph registry or lock is reachable from the inspection. The private
receipt and lookup bindings still contain aliases; Clone and Send/Sync on those
inputs are not isolation evidence. No unsafe Send/Sync implementation is added.

## Scope, freshness and limitations

Reuse existing copier, history composition functions, actual-context traversal,
frozen naming, capture and final rebind. No modeling/evaluation engine or repair
fallback is added. Capture lineage before invoking the existing freeze entry;
after its final external callback, enumerate and project the selected/copied
contexts, observation rows, parents, relations, sweep roles and retirement again.
Compare exact owned lineage bytes. The frozen projection comes from the same
receipt and authored operation bindings. No callback follows final comparison.
Concurrent writes through input aliases are outside this operation-local Interface.

The fixed existing 4096-record capture bound applies to lineage observations and
additional bindings. Excess is refused without truncation. Cancellation uses the
existing freeze checkpoints; bounded provenance projection itself adds no external
callback. The existing frozen geometry domain and capture policy are unchanged.

Historical selected geometry remains unproven. This slice does not import region,
embedding, measurement or tolerance proofs. Upstream partition provenance and
original deletion composition remain explicit limitations. Retiring a copied
wire is retained separately from original deletion; `ComposedDeletionCoverage`
is not removed. Rejected/unbound history and whole-body validation limitations
remain in the frozen projection. Stable split/merge naming for upstream partition
intermediates still requires complete explicit source identities and provenance.

## Version, replay and deletion gate

This is an additive private Interface in an unpublished diagnostic path, with no
released consumer or package version change. New inspection behavior is version 1;
the existing receipt and frozen capture remain independently selectable. Baseline
commit `83d04124b` pins the unchanged prism tests and 89,921-byte frozen replay.
Structural observation retention is separate (`8e5c31b24`). No old expected value
or behavior fixture is updated to admit this inspection.

Deletion task: remove redundant flattened-only provenance transport at the eventual
publication seam after complete operation/copy/partition/healing history has explicit
stable identities and full ordered validation. Migration limit: the region-to-solid
milestone. Until that gate passes, preserve the diagnostic path and typed gaps.
The gate additionally requires source/failure/cancellation isolation, independent
invariants, deterministic process/thread replay and sanitizer acceptance. Language
Send/Sync and deterministic stress are separate from unverified sanitizer runtime
acceptance; the prior host TSan control startup failure is not a waived gate.
