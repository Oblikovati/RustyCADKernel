# Sensitive entity set and shared BVH continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Source changes remain uncommitted:
read-only Git metadata prevents integration into main. Preserve this checkout and
the original Claude worktrees until integration can finish.

The [fresh inventory](METRICS.md) accounts for **51,146/76,041 defined APIs
(67.3%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes reasoned exclusions and does not prove full behavioral
parity. This follows the [ranking checkpoint](../selection-ranking-continuation/README.md).

SelectMgrSensitiveEntitySet preserves indexed ordering, swap-last removal, owner
frequencies, duplicate/current-flag persistence counters and native runtime casts.
BvhTreeHandle retains shared identity across rebuild, set destruction and partial
builder failure. Primitive-set tree access now returns a read guard; callers must
release it before mutating the set. Existing traversal adapters copy their working
tree to avoid temporarily emptying retained handles.

Verification: **3,926 Rust tests** across five affected crates (10 new), including
174 TKV3d tests; **126,524 TKV3d native comparisons** (608 new), **75,190 TKMath
comparisons**, one native and one Rust mapped legacy case. Workspace all-targets
check, TKV3d Clippy with warnings denied, seven corpus-wiring tests, formatting and
diff checks passed. All 17 new API mappings resolve uniquely; the refreshed
crosswalk has zero unresolved rows. Initial broad Rust runs emitted guard-use
warnings; the affected call sites were corrected and tested again. Two new test
clone expressions were simplified after the broad run and all TKV3d tests reran.

[receipt.json](receipt.json) records hashes and counts. [continuation.patch](continuation.patch)
preserves all source changes relative to the parent and passes reverse checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [TKV3d runtime report](../../../runtime-evidence/tkv3d-sensitive-entity-set-2026-10-09.json.gz)
and [TKMath report](../../../runtime-evidence/tkmath-sensitive-entity-set-2026-10-09.json.gz)
record the validation. Historical reports remain unchanged.

[next-selectable-object-set-notes.md](next-selectable-object-set-notes.md) records
native behavior and existing interfaces for the next set. The original unfinished
TKernel linked-slot refactor remains untouched and unvalidated. The port is incomplete.
