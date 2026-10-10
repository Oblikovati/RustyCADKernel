# Viewer selector traversal and public picking continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main at checkpoint creation. Git write access was restored after restarting the session;
integration follows this checkpoint. Preserve
this checkout and all original Claude worktrees. Twenty-five committed continuation
commits remain outside main; the original dirty TKernel linked-slot refactor is untouched.

The [fresh inventory](METRICS.md) accounts for **51,412/76,041 defined APIs
(67.6%)** in the unchanged 56-toolkit scope. This slice adds **seven
implemented declaration mappings**: precise overlap matching, object traversal,
root traversal and all four public Pick overloads. Every new mapping resolves uniquely;
unresolved ledger rows remain zero. Done includes prior exclusions and does not
establish complete behavior parity. Counts describe the continuation, not main.

The selector now traverses real object and sensitive BVHs across all five persistence
subsets, applies clipping, transformations, flipping, view affinity and owner-complete
area containment, and dispatches derived sorting. Public picks preserve volume reuse
and native camera/tolerance initialization. Callback failure retains partial results and
releases the worker-pool sentry. Overlap processing preserves native live owner-slot
replacement timing. The headless native driver now retains clipping planes, matching
the Rust driver and making public-pick clipping comparisons effective.

Checks passed: **271 TKV3d Rust tests**, **132,707 native comparisons**, one native and
one Rust mapped legacy case, Clippy with warnings denied, seven tooling tests, formatting
and diff checks. Fourteen direct tests and 1,408 numeric cases are new: 192 overlap
traces, 1,042 traversal cases and 174 repeated public-pick traces. The existing native/Rust
SIGABRT check for virtual Clear failure during context destruction also passed.

ViewerSelector remains partial: DisplaySensitive and ToPixMap are still open. Camera
handle replacement inside a matching callback remains an unverified reentrancy edge;
[continuation notes](next-context-notes.md) describe it and the next dependency chain.
The implementation uses a checked NullObject for a missing view window. Valid polyline
fixtures use one-based indexing because the native orientation loop assumes it.

[receipt.json](receipt.json) records counts and source hashes. [continuation.patch](continuation.patch)
contains all changes relative to the continuation parent and passes reverse checking;
generated inventory binaries are omitted and must be regenerated after integration.
Historical checkpoints remain unchanged.
