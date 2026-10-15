# ADR 0058: Bounded complete annular prism history

Status: private V2 issuer implemented on a worktree branch (2026-10-15, see
"V2 issuer" below); main integration and the root gate remain the coordinator's.
V1 remains selected by its original entry; its types gained only one additive
legacy cause variant (`AnnularJoin`) and wider helper visibility.

## V2 issuer (2026-10-15)

`frozen::annular_history::inspect_annular_prism_history` (plus
`inspect_bound_…`, `rebind_…`, `reproject_…`) consumes the same
`TwoLoopRegionRequest`, `PrismReceipt`, publication/copy bindings, explicit
`AnnularHistoryBindings` scopes and the sealed `AnnularFrozenDomain`.

- Scopes: `Original(source)`/`InputCopy(source)` per authored source,
  `Partition`, `CandidateCopy`, `Selected(name)`/`Rejected(name)` per authored
  candidate, `SweepBase`. The explicit roster must be exactly both InputCopy
  scopes, Partition, CandidateCopy and every rejected candidate; duplicates,
  extras, stage mislabels and member names reused across scopes refuse.
- Partition queries are resolved per source through that source's own initial
  copier image; partition relation parents are derived from each original's
  actual continuation (root container: generated from all originals), never by
  shape kind. Empty Modified/Generated and false Deleted answers are retained.
- Every candidate binds its sealed disposition, authored wire roles, actual
  generated area (ordinal re-checked) and its receipt; rejected receipts bind
  one-to-one. Each copied use traces into the annulus and exactly one rejected
  domain whose wire has the same authored source.
- Sweep answers reuse the exhaustive V1 projection; captured flat parents bind
  the same full source/output/role native row, keep captured order and equal
  the native class union. Six original continuations reach both caps; the two
  copied annulus wire retirements are recorded separately; deletion is empty
  only after all continuations.
- Admission charges authored scopes, native operands and live-root censuses
  before projection; terminal callback precedes the shared join rebind and a
  callback-free full-byte reprojection. Join refusals keep their carrier in
  `AnnularHistoryFailure::join`.

Evidence: 28 `annular_history` tests (operation, adversarial, native refusal,
ten-geometry variant corpus incl. stored/traversed reversal, reflection,
negative sweep and reordered request rows with identical replay, two-thread
equality). This history still grants no body validity.

The proposed inspection would consume the same retained two-loop request, actual sweep
receipt, authored publication/copy/intermediate bindings and sealed named annular
join. Its scoped rosters distinguish source-specific original/input copies and
authored candidate-specific selected/rejected regions. Neither parent equality,
returned order, coordinates nor native addresses supplies identity.

Reuse existing context/copy/partition/sweep projections. Retain exact actual
query results, including empty answers, false retirement and errors, all relation
classes and parent multiplicities, and captured flat parent order on the same
full source/output/role row. Every original directed use must have a proven
continuation or deletion; intermediate wire retirement is not original deletion.
Missing required coverage remains incomplete. There is no raw success constructor.

Final callbacks precede the shared live annular join rebind and history-only
operand reprojection, without callbacks between those checks. The owned result
contains immutable values and full replay; source-local diagnostics retain native
causes separately when they cannot cross that ownership boundary. Bounds are
checked before allocation, with requested/effective/measured structural units.

This history proof does not publish a valid body. Full ordered validation,
measurements and owned-request integration remain downstream gates. Within
Kernel #10, delete temporary fixture adapters after one complete named annular
operation passes its independent corpus; retire overlapping private orchestration
within one milestone only through recorded migration preserving V1 replay. This
unreleased additive private API does not change the workspace version.

## Structural checkpoint at wrap-up

Source `b6dc60301` adds actual two-source/three-candidate native-census fixtures and
shares existing V1 projection helpers through narrow operation-local borrowed
inputs. It has no annular history request/result/issuer. The coordinator reviewed
the shared-input extractions and preserved V1 wrappers without a semantic blocker.
The user paused broader work and requested current workers finish and hand off;
the coordinator explicitly chose this structural stopping point before V2 behavior
implementation began.

[Final owner gate](../evidence/annular-complete-history-v2-2026-10-14/FINAL.md) and
[deferred implementation handoff](../evidence/annular-complete-history-v2-2026-10-14/HANDOFF.md)
record the actual completed scope, exact replay preservation and remaining proof
obligations. Main integration and capability acceptance remain distinct.
