# Selecting volume manager continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Source changes remain uncommitted:
read-only Git metadata prevents integration into main. Preserve this checkout and
the original Claude worktrees until integration can finish.

The [fresh inventory](METRICS.md) accounts for **51,105/76,041 defined APIs
(67.2%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes reasoned exclusions and does not prove full behavioral
parity. This follows the [frustum set checkpoint](../triangular-frustum-set-continuation/README.md).

SelectMgrSelectingVolumeManager now implements active-volume reuse, configuration,
copy/rebuild and transforms, all overlap queries, clipping and native sentinels.
SelectBasics distance/ray getters and Select3D matching propagate intersector errors.
The manager preserves delayed polyline overlap-flag updates, stale clip ranges
outside Point, and differing empty-manager copy/transform behavior. GetVertices
returns None for nonrectangular selectors where the native static_cast is undefined.

Verification: **159 Rust tests** (10 new), **96,090 native comparisons** (4,800 new),
one native and one Rust mapped legacy case, Clippy with warnings denied, seven
corpus-wiring tests, formatting and diff checks. All 52 new API mappings resolve
uniquely; the refreshed crosswalk has zero unresolved rows.

[receipt.json](receipt.json) records hashes and counts. [continuation.patch](continuation.patch)
preserves all source changes relative to the parent and passes reverse checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [runtime report](../../../runtime-evidence/tkv3d-selecting-volume-manager-2026-10-09.json.gz)
is bound to the tested source. Historical reports remain unchanged.

[next-selection-ranking-notes.md](next-selection-ranking-notes.md) describes the
next tolerance/ranking dependencies. The original unfinished TKernel linked-slot
refactor remains untouched and unvalidated. The overall port is incomplete.
