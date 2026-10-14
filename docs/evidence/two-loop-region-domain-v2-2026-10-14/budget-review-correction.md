# Draft review correction: retained roster admission

Independent read-only review of fixed `99a107f89` found that source matrix entries,
construction/parent rosters and census snapshots could be walked by legacy binding
inspectors before their structural-unit budget was admitted. The original source
and 32-test receipt remain retained; that receipt did not establish bounded admission
for these extra containers.

The correction charges the full borrowed source matrix (including retained entries,
stages, faults, original/construction curve poles/weights, segment and kinded relation
parents), paired occurrence/report source rosters, and candidate metadata before
legacy source/domain/history inspectors allocate correspondence or fault lists.
Partition space descriptors are also charged. History preadmission now precedes the
source/pair inspectors. Actual stored evaluator pole admission is repeated after the
Census callback, before legacy geometry recapture.

Units count structural records, pole vectors, weights, and parent entries, not bytes
or machine instructions. Borrowed validation input rows and separately owned output
rows are each charged at their consumption boundary; this deliberately includes
repeated captures. Exact integers' limb storage and immutable placement internals
are not a heap-byte quota. Existing solver bisection and pole limits remain separate.
Opaque mathematical reports are immutable provider outcomes; their source roster
is preadmitted, and each exposed proof array is charged before projection allocation.
No existing V1 path or replay format changed.

Executed on unchanged correction source: Rust1.99, jobs2,
`cargo +1.99.0 test -p rustycad-tkbo --all-features two_loop -- --nocapture`.
35 tests passed (34 two-loop tests plus one matching legacy topology test), including
exact/one-short/tiny source budget, injected retained source matrix, nested paired
parent/coefficient accounting, whole-operation exact/short budget, all cancellation
cutoffs, process/thread comparisons, and final live rebinding. See the compressed
complete log and source digest manifest. This is focused evidence, not the pending
full candidate acceptance gate. The independent correction re-review is pending.

## Follow-up: callbacks during exact chart-rank evaluation

Independent re-review accepted the nested-roster accounting at `a75f005ed` but
identified an ordering hole: `plane::inspect` invokes external progress after the
second evaluator admission. Its callback could grow a curve before pair binding
recaptures live geometry. A new regression replaced a four-pole original with five
poles during the rank callback with requested cap four. The red run failed the
expected typed `Limit` assertion (the later stale-binding check refused instead).
The correction moves evaluator admission immediately after rank evaluation, with
no external callback between admission and pair current-binding checks.

The full focused run now passes 41 tests (40 two-loop plus one matching legacy),
including the new rank mutation regression. Original red and full green logs are
retained. This checkpoint also preserves native/cancelled classes through owned
and local typed cause wrappers, rejects a failed/cancelled arrangement even if a
candidate batch remains, retains owned pair-capture failures, distinguishes stale
complete reports, and records signed domain/material winding conclusions per
candidate. Those additions change only the unpublished V2 proof bytes; its complete
draft payload is now pinned before later structural cleanup. V1 paths are unchanged.

Independent read-only re-review of fixed `aa8a7b3a4` accepted both reported budget
corrections: full nested-roster admission and callback-free evaluator admission
immediately after rank. Reviewer ran no tests; broader added refusal classification
was explicitly outside the narrow correction approval. Final integration gates remain
separate. Additional adversarial tests pass (46 focused total): coincident distinct
sources, disjoint unnested native closure, extra genuine generated area/missing member,
separate same-parent authored keys, and duplicate parent refusal. An initial test-only
compile failure attempting Clone on an owned native area is retained; the corrected
test inserts a genuine independently built extra area. Full V2 bytes match the prior
pinned draft exactly. This is the final pre-cleanup test corpus checkpoint.
