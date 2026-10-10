# Viewer selector state continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main because session filesystem policy makes Git metadata read-only. Preserve
this checkout and all original Claude worktrees. The original dirty TKernel linked-slot
refactor on `worktree-agent-aeed1901b085eca95` remains untouched and unvalidated.

The [fresh inventory](METRICS.md) accounts for **51,258/76,041 defined APIs
(67.4%)** in the unchanged 56-toolkit scope. This checkpoint adds
64 implemented mappings and one explicitly partial ViewerSelector class. Done includes
prior exclusions and is not complete behavior parity. Counts describe the continuation,
not main. Every new mapping resolves uniquely; unresolved ledger rows remain zero.

The selector now preserves native state defaults, protected fields, virtual lazy sorting,
borrowed picked criteria and entity handles, retained index storage, membership/status,
active-owner queries, real BVH pool controls, direct owned manager/set access and JSON
dumps. The shared libstdc++ sort now propagates comparison failures with native partial
moves, including the heap fallback. Pick/traversal, layer collection, image/debug display
and friend activation orchestration remain unfinished; the class is explicitly partial.

Checks passed: **4,937 TKernel and 209 TKV3d Rust tests**, **127,416 TKV3d native
comparisons**, one native and one Rust mapped legacy case, **75,190 TKMath numeric
comparisons**, **9,368 sorting failure/success trials across eight shapes**, Clippy with
warnings denied, seven tooling tests, formatting and diff checks. Twelve direct tests
and 170 native selector traces are new. The full TKernel runtime gate requires a clean
checkout and was not rerun; its safeguard remains unchanged. The source-bound focused
STL audit is recorded separately, not advertised as full TKernel acceptance.

[receipt.json](receipt.json) records counts and hashes. [continuation.patch](continuation.patch)
contains all source changes relative to the continuation parent and passes reverse
checking; generated inventory binaries are omitted and must be regenerated after
integration. Historical evidence remains unchanged. [next-traversal-notes.md](next-traversal-notes.md)
records native algorithm ordering and the missing AIS/context dependency discovered
while preparing the next porting step.
