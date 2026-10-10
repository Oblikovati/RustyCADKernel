# Selection manager continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main because session filesystem policy makes Git metadata read-only. Preserve
this checkout and all original Claude worktrees; the original dirty TKernel linked-slot
refactor on `worktree-agent-aeed1901b085eca95` remains untouched and unvalidated.

The [fresh inventory](METRICS.md) accounts for **51,277/76,041 defined APIs
(67.4%)** in the unchanged 56-toolkit scope. This adds 19 implemented
mappings and one partial BRep selection-tool record. Done includes prior exclusions,
not complete behavior parity. Counts describe the continuation, not main. Every new
mapping resolves uniquely and unresolved ledger rows remain zero.

SelectionManager now provides its complete public/protected orchestration: retained
selector access, child-first lifecycle, activation, mode queries, update/recompute paths,
structure restoration and sensitivity changes. Private mode loading and foreground/
background BVH decisions are translated too, with real ViewerSelector friend activation.
StdSelectBrepSelectionTool ports only PreBuildBVH; shape construction helpers remain open.
ViewerSelector picking/traversal and the wider AIS context remain incomplete.

Checks passed: **219 TKV3d Rust tests**, **127,716 native comparisons**, one native and
one Rust mapped legacy case, Clippy with warnings denied, seven tooling tests, formatting
and diff checks. Ten direct tests and 300 numeric cases are new, covering hierarchy,
callback failures, partial mutation, inactive recomputation, foreground thresholds,
nested group handling and actual background completion.

The native oracle crashes after Activate(unloaded), Remove, then Activate of another
mode: removal skips selector cleanup outside global membership. The [minimal input](native-undefined-sequence.txt)
and diagnostic stack are preserved. The corpus tests unloaded activation separately
and establishes membership before later randomized activation lifecycles; it does not
assign a Rust parity result to native undefined memory access. Protected presentation
ownership is configured before the lifecycle; callback failures remain covered.

[receipt.json](receipt.json) records counts and hashes. [continuation.patch](continuation.patch)
contains all source changes relative to the continuation parent and passes reverse
checking; generated inventory binaries are omitted and must be regenerated after
integration. Historical evidence remains unchanged. [next-ais-notes.md](next-ais-notes.md)
records the AIS selection/status/context dependencies and remaining viewer traversal.
