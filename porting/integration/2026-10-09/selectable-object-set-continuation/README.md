# Selectable object set continuation

The tested source is `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Source changes remain uncommitted:
read-only Git metadata prevents integration into main. Preserve this checkout and
the original Claude worktrees until integration can finish.

The [fresh inventory](METRICS.md) accounts for **51,173/76,041 defined APIs
(67.3%)** in the unchanged port scope. These are continuation counts,
not main counts. Done includes reasoned exclusions and does not prove full behavioral
parity. This follows the [sensitive-set checkpoint](../sensitive-entity-set-continuation/README.md).

SelectMgrSelectableObjectSet implements all five indexed subsets, live iteration,
swap-last removal, stable shared trees, camera/projection/viewport invalidation,
orthographic copies, persistent group bounds and native dump fields. Value copies
have independent maps and flags but share tree and builder handles. Empty subsets
retain old trees; failed updates retain native partial changes and dirty flags.
Flipping uses WorldView*ModelWorld and deliberately excludes group-local transforms.
The headless fixture supplies the selection metadata normally created by OpenGl_Group;
no rendering behavior is claimed.

Verification: **186 Rust tests** (12 new), **127,066 native comparisons** (542 new),
one native and one Rust mapped legacy case, Clippy with warnings denied, seven
corpus-wiring tests, formatting and diff checks. All 27 new API mappings resolve
uniquely; the refreshed crosswalk has zero unresolved rows.

[receipt.json](receipt.json) records hashes and counts. [continuation.patch](continuation.patch)
preserves all source changes relative to the parent and passes reverse checking.
Generated inventory binaries are omitted and must be regenerated after integration.
The [runtime report](../../../runtime-evidence/tkv3d-selectable-object-set-2026-10-09.json.gz)
is bound to the tested source. Historical reports remain unchanged.

[next-bvh-thread-pool-notes.md](next-bvh-thread-pool-notes.md) records the next pool's
native behavior and existing thread/event/sensitive interfaces. The original unfinished
TKernel linked-slot refactor remains untouched and unvalidated. The port is incomplete.
