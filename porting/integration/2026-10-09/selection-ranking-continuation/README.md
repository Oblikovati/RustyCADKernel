# Selection tolerance and ranking continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Source changes remain uncommitted:
read-only Git metadata prevents integration into main. Preserve this checkout and
the original Claude worktrees until integration can finish.

The [fresh inventory](METRICS.md) accounts for **51,129/76,041 defined APIs
(67.2%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes reasoned exclusions and does not prove full behavioral
parity. This follows the [manager checkpoint](../selecting-volume-manager-continuation/README.md).

SelectMgrToleranceMap preserves active counts, retained zero-frequency entries,
custom offsets and resets. SelectMgrSortCriterion exposes the native candidate
record and both ranking predicates, including native normal arithmetic, priority
inversions, confusion thresholds and adjustment by tolerance.

Verification: **167 Rust tests** (8 new), **125,916 native comparisons** (29,826 new),
one native and one Rust mapped legacy case, Clippy with warnings denied, seven
corpus-wiring tests, formatting and diff checks. All 24 new API mappings resolve
uniquely; the refreshed crosswalk has zero unresolved rows.

[receipt.json](receipt.json) records hashes and counts. [continuation.patch](continuation.patch)
preserves all source changes relative to the parent and passes reverse checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [runtime report](../../../runtime-evidence/tkv3d-selection-ranking-2026-10-09.json.gz)
is bound to the tested source. Historical reports remain unchanged.

[next-sensitive-entity-set-notes.md](next-sensitive-entity-set-notes.md) records the
next BVH set's behavior and its missing handle/RTTI interface. The original unfinished
TKernel linked-slot refactor remains untouched and unvalidated. The port is incomplete.
