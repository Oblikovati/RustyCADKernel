# BVH thread pool continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata remains read-only;
changes are uncommitted and not integrated into main. Preserve this checkout and
all original Claude worktrees. The original dirty TKernel linked-slot refactor on
`worktree-agent-aeed1901b085eca95` remains untouched and unvalidated.

The [fresh inventory](METRICS.md) accounts for **51,194/76,041 defined APIs
(67.3%)** in the unchanged 56-toolkit scope. This adds 23 implemented
mappings and two explicitly prohibited, declared-but-undefined native sentry copy
exclusions: **21 defined APIs plus four separately tracked special-member rows**. Done includes exclusions and is not a claim of complete behavior parity.
These counts describe the continuation, not main.

The pool uses real OSD workers, retained duplicate jobs, lazy startup, native events,
stop/restart with queued jobs, failure recovery, captured signal setup, and mutable
worker controls. Copies share thread handles and retain distinct BVH mutexes; an
owning noncopyable sentry locks workers in array order. OSD now accepts owned callback
context and permits inspection during joins, preserving copy/detach semantics.

An empty-queue reset could erase shutdown's wake in both the initial Rust translation
and pinned native code. Rust synchronizes that ordering and has a deterministic
regression plus 100 restart cycles. The unchanged native oracle parks its workers
inside barrier-controlled jobs before shutdown to avoid that native race. Counts
are observed at that barrier because native WaitThreads can miss a job between
queue removal and worker mutex acquisition. The Rust implementation preserves that
native ordering. Hardware-signal-to-exception conversion retains the existing OSD
platform limitation; no unsupported signal conversion is claimed.

Checks passed: **4,936 TKernel and 198 TKV3d Rust tests**, **127,246 TKV3d native
comparisons**, one native and one Rust mapped legacy case, **251 focused OSD thread
comparisons**, Clippy with warnings denied, seven tooling tests, formatting and diff
checks. Eighteen direct tests and 180 numeric traces are new. The full TKernel runtime
verifier rejects this dirty checkout; its safeguard remains unchanged. The focused
thread validation is source-bound and recorded separately, not advertised as full
TKernel acceptance.

[receipt.json](receipt.json) records hashes and counts. [continuation.patch](continuation.patch)
contains all source changes relative to the continuation parent and passes reverse
checking. Generated inventory binaries are omitted and must be regenerated after
integration. Historical checkpoints and reports remain unchanged. The full port is
still incomplete; [next-selector-notes.md](next-selector-notes.md) records ViewerSelector
and selection orchestration work.
