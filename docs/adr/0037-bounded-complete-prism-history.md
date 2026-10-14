# ADR 0037 — Bounded complete history for one Jordan prism

Status: private candidate; final acceptance evidence is recorded separately.

## Decision and boundary

Martensite composes actual existing copy, partition, candidate generation and
sweep observations for one admitted full-range polynomial Bézier Jordan curve.
The implementation is a private inspection of an existing operation, not a
second modeling or naming engine. It does not construct a body, replace the
existing builders, or change `PrismReceipt` completion or its
`ComposedDeletionCoverage` gap. The ordered publication assembler owns acceptance.

Behavior V1 admits one unsplit original edge and its two directed uses of one
closure vertex. It requires a two-region complementary candidate universe,
exact original/copy/partition context censuses, complete native partition
queries with empty Modified/Generated and false deletion, and exhaustive sweep
queries. Rational, split, multi-edge and general Boolean histories remain
explicitly incomplete. There is no fallback, truncation, reconstructed identity,
implicit healing, or inferred valid solid.

All production additions are crate-private under the existing 0.2.7 package.
No released public consumer or signature changes; no package version bump is
required. V1 frozen and copy-lineage APIs, classifications and replay remain
selectable and unchanged. Their owned fixtures predate this work.

## Interface and ownership

`PrismHistoryRequest<'inputs, 'join>` borrows the actual bridge, source shapes,
receipt, existing publication/copy names, additional history names, an existing
`JoinedPrismProof`, and explicit history options. The join lifetime is separate:
operation-local refusal diagnostics never borrow a caller's local join.

`inspect_complete_prism_history` admits inputs, rebinds the shared join, projects
history, then after its last callback repeats shared live rebind and complete
history projection. `rebind_complete_prism_history` provides that standalone
final check without callbacks. `inspect_bound_prism_history` is a conditional
callback-free projection for the assembler's freshly joined inputs; it does not
perform a second shared join evaluation. The assembler must subsequently run
one shared live join rebind and `reproject_complete_prism_history`, with no
callback between or after them. The latter reprojects all extra history
operands, not just the body's joined bytes.

Outcomes are Inspected, Incomplete, Failed and Cancelled. Inspected owns all
history keys, scalar snapshots, parent multiplicities, query outcomes,
continuations, requested/effective options, measured operand rows and complete
joined replay bytes. Its borrowed views have no mutation interface. No Shape,
Handle, Report, cache, registry or context address survives in successful
history. `Send + Sync` is asserted for this evidence and owned findings. Neither
`Clone` nor an immutable reference to the legacy receipt is an isolation proof.

The conditional entry alone does not prove that the supplied join is current;
its documented final shared rebind obligation is mandatory before publication.
History also does not independently prove geometry, trims, injectivity or
ordered body validation. Those proofs remain in the shared pipeline.

## Explicit identity and correspondence

Operation, selected region and closure contact come from existing authored
bindings and the joined proof. The caller additionally names the complementary
region and every physical member/directed use of InputCopy, Partition,
CandidateCopy and Complement. Original, Selected and SweepBase reuse the
existing names. Bindings must be bijective; missing, duplicate, extra or
ambiguous use attribution refuses inspection.

Actual process-local member/context identities and temporary tree/DS/area
indices are lookup operands only. Published keys are authored stage/member/use
names with complete authored ancestry. Neither parent sets, coordinates,
returned order nor addresses create identity. Two members with identical
original parents remain distinct. Child order is retained as ordered authored
child keys; it does not manufacture a key.

Actual per-context copier Modified queries establish unique member maps at
every ancestor, exact placement and orientation. Generated boundary edges reuse
actual copied members. Endpoint use correspondence uses relative F/R orientation
under the actual edge, with complete ancestor census and uniqueness; it never
uses proximity or sibling rank. Native source endpoint parameters are bound to
that actual relative directed role on the admitted full `[0,1]` interval.

Every relation class retains its actual parent multiplicity. Expected legacy
container unions may use the existing set semantics, but actual observations
are never deduplicated to hide mismatches. Actual selected and rejected receipt
relations must equal the corresponding retained batch identities and full
parent lists. The administrative copied compound omitted by generation is an
explicit intermediate disposition, not original deletion.

## Queries, deletion and bounds

Structural observation commits retain every actual partition/sweep query,
including available-history state, empty answers and false retirement. Absent
history remains unknown. A true sweep retirement of the copied wire is retained
separately, even when its First/Last cap answers are non-null. Empty original
deletion is concluded only after every actual original directed use has a
complete initial-copy/partition/candidate-copy/selected/sweep-base chain into
kept cap uses. Complement rejection is accounted independently.

Sweep query coverage compares the exact multiset of authored copied query use,
role, directed output context and every native parent class against actual
bindings. Matching counts or sets are insufficient. One native answer can
correspond to several directed uses of the same physical output, all retained.

Requested V1 defaults are depth 32, directed uses per root 4096, and 16384
stored history operand units. Requests above hard bounds 64, 16384 and 65536,
respectively, are refused. Zero is refused. No clamp changes effective settings.
All count addition is checked for overflow. Ancestor TShape identity, independent
of changing placement, detects cycles before legacy traversal.

The row budget counts each supplied/retained vector row consumed by this
history projection: authored binding rows, context rows and their ancestor/path
entries, copier/query/image rows, each occurrence of a parent in each relation
class, tree/area/generated rows and descendants/copy indices, partition source,
vertex/endpoint/block/image rows and variable Bézier coefficient/weight rows.
Nested native returned relation parents, copied observation parents and directed
binding context operands are included. Repeated observations are counted each
time. This is a structural operand count, not a byte or execution-work estimate;
fixed-size scalar fields and the already-budgeted shared join replay block are
not charged per byte. The measured total is owned and replayed. Refusal reports
the first checked exhausted total; no partial successful evidence is returned.

## Diagnostics and replay

Owned findings retain operation, phase, known authored use, failure class and
native typed cause. Legacy owner-bearing join/candidate/partition/arrangement
refusals remain in a separate operation-local diagnostic envelope. The owned
finding identifies the exact stage/outer variant and safe scalar/count/native
payload; unsupported ownership is not erased or turned into success. Known
native/failed source causes are Failed. A previously cancelled prerequisite is
Incomplete with its original typed cancellation preserved; only cancellation
of this inspection's callback returns its Cancelled outcome.

Complete replay contains all authored contexts, copied observations, metadata,
query outcomes, original parent classes, selected/complement dispositions,
intermediate retirement, kept original continuations, effective limits and full
joined bytes. Final reprojection compares these exact bytes. Concurrent alias
mutation during callback-free reads is outside this private contract; callbacks
are supported mutation boundaries and are checked after the last call.

## Evidence, review and migration

Independent expectations are combinatorial: two original physical definitions,
three original directed uses, two complementary regions, one administrative
candidate-copy compound, five sweep-base uses, one copied-wire retirement and
zero original deletions after all three original continuations. They use owned
analytic fixtures and actual native observations; there is no live OCCT oracle.

The fixed-source independent review of d9daef1bc found three blockers. Its
findings, retained red tests, correction revisions and re-review status are in
`docs/evidence/prism-complete-history-v1-2026-10-14/independent-review.json`.
Earlier failed assumptions and logs remain retained. Structural cleanup must
pass the complete operation/cancellation/refusal corpus and reproduce both
legacy replay byte strings and the corrected complete-history baseline.

Ordinary-host TSan runtime acceptance remains unverified. The previously
recorded instrumented trivial control failed before its main marker; exact
commands/toolchain are in the frozen-prism evidence TSan metadata. No host
security setting or runtime check is bypassed. Language-enforced Send/Sync and
successful deterministic thread/process checks are separate evidence.

Deletion task for Kernel#10: once the ordered named-prism assembler passes the
full accepted-operation corpus, remove temporary duplicate legacy history
retention from its successful publication path, while keeping explicit replay
selection for prior receipts. Corpus gate: exact per-use history, all parents,
zero unknown required dispositions, source isolation, full ordered checks,
independent measurements and deterministic process/thread bytes. Migration is
limited to Kernel#10; no broader history replacement is authorized by this ADR.
