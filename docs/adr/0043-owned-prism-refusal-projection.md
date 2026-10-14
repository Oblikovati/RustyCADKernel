# ADR 0043: private owned prism refusal projection V1

Status: additive private implementation prefix; complete public operation refusal corpus pending.

## Decision

RustyCADKernel#10 needs an immutable operation outcome containing actionable typed
refusals as well as successful geometry. A `Send + Sync` payload can still carry
process-address identities; those are a separate obstacle to a public contract.
This prefix adds one optional projection over the existing join failure. It does
not change a legacy operation, exception, classification, or stage precedence.
The existing local failures remain available unchanged without invoking it.

`project_join_refusal` consumes a join failure, the caller's original operation
class, and a borrowed identity context. It returns either an immutable V1
`OwnedPrismJoinRefusal`, or projection-incomplete evidence **and the complete
original local failure**. Projection completeness never overwrites operation
Failed/Incomplete/Cancelled. No partial geometry or body validity accompanies a
projected diagnostic. No public exports, workspace version, or release changes
are introduced.

Original-member identities use the existing exact authored-key lookup. Missing,
ambiguous, zero-token and reverse-colliding bindings refuse projection. Addresses
are lookup operands only. Existing full frozen keys and ancestry remain intact;
no coordinate/rank naming or generic face-token substitution is introduced.
Authored tokens are meaningful within the bound operation; the complete caller
semantic dictionary from ADR0042 is still required for a public envelope.

## Supported prefix and explicit gaps

- Every join outer variant is distinguished. Existing frozen capture/link causes
  remain owned typed evidence, including their nested numerical witnesses.
- Source conformance scalar causes move intact; MissingOriginal resolves to the
  complete authored original key. Census/segment observation numbers are retained
  as observation metadata, not persistent entity identities.
- Region hypotheses, representation captures and native exceptions are retained.
- Embedding options, phase, original key, regularity coefficients/root witnesses,
  actual work, polynomial weights and nonregular reports are retained.
- Binding/chart/body controls, invalid scalar requests, native exceptions,
  classification numerical failures and callback-plus-classification failures
  retain their distinct typed payloads. Native Failure values move unchanged,
  preserving non-UTF8 bytes, exception parent and stack.
- Closure-point contexts, candidate report/gap/captured-store payloads, body
  selection and shape-bearing body causes are explicitly projection-incomplete.
  Their full original objects and nested scalar evidence are retained locally.
  They are not replaced by strings, counts, or invented entity names.

Arbitrary alert projection is not claimed. The existing frozen alert projector
recognizes the dataless historical IntersectionFailed-at-Fail case only. New
report payloads require concrete typed adapters and the complete reachable corpus;
unknown payloads remain a public-release blocker. No diagnostic failure is erased.
Joined measure side causes also require source/output identity projection despite
being owned already; this prefix does not export those types automatically.

## Integration and migration

The publication owner keeps local diagnostics internal until it can invoke this
projection on every reachable refusal. History's compact legacy summaries cannot
replace their separate full local payloads. Ordered-validation and measurement
failures retain their precise scalar causes and existing classification.

Kernel#10's operation-contract owner owns the existing migration/deletion task:
finish the bounded complete-operation refusal corpus and caller dictionary join,
then remove superseded lossy public adapters if any are introduced. Keep replay
of the original local behavior until that recorded migration. ADR0014's existing
application R3 limit applies; this prefix does not create a new milestone or claim
C1, application acceptance, or the sketch-with-hole gate.

## Verification boundary

Baseline commit 8de45ff72 pins native bytes and failed/incomplete/cancelled local
variants before production projection. Prefix checks cover identity refusal,
nested payload retention, exact work/options, source-owner drop, and independent
thread results. Compile-time Send/Sync assertions apply to the projected values,
not the local incomplete envelope. Ordinary host concurrency is distinct from
ThreadSanitizer: the previous host startup failure does not count as execution of
this new suite. Full publication corpus, process replay, TSan and release checks
remain pending unless separately recorded by a subsequent frozen receipt.
