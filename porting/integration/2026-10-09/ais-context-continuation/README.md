# AIS context and interactive-object continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main because session filesystem policy makes Git metadata read-only. Preserve
this checkout and all original Claude worktrees. Twenty-five committed continuation
commits remain outside main; the original dirty TKernel linked-slot refactor is untouched.

The [fresh inventory](METRICS.md) accounts for **51,405/76,041 defined APIs
(67.6%)** in the unchanged 56-toolkit scope. This slice adds **88 implemented
declaration mappings** and one partial context-class mapping. The API total increases
by 87; the implemented destructor is classified separately as a special member. Done includes prior exclusions,
not complete behavior parity. Counts describe the continuation, not main. Every new
mapping resolves uniquely; unresolved ledger rows remain zero.

AIS context construction uses real presentation and selection managers, a polymorphic
selected-owner container, actual viewer links and all six native drawing styles.
Highlight plane-edge graphics intentionally share the original default aspect handle.
The abstract interactive object now supplies context/application-owner access, real
presentation lookup, redisplay, supported last-group aspect updates, virtual defaults
and the AIS dump override. Context recomputation preserves callback order, partial
failure state and recorded selection-mode reactivation. Selection replacement preserves
virtual Clear dispatch during context destruction. LastActiveView checks live viewer
membership. The native uninitialized initial raw view pointer is safely null in Rust;
the native fixture initializes it explicitly before reading.

Checks passed: **257 TKV3d Rust tests**, **131,299 native comparisons**, one native and
one Rust mapped legacy case, Clippy with warnings denied, seven tooling tests, formatting
and diff checks. Fourteen direct tests and 570 numeric cases are new: 145 constructor/style
traces, 417 object update/failure/aspect/dump/view traces, and eight selection replacement
and shutdown cases. Only process-local dump pointer text is normalized; identities are
compared separately. An isolated native/Rust comparison and regression test require
SIGABRT when virtual Clear fails in the noexcept context destructor. Rendering and selection callbacks remain abstract as in OCCT.

AIS_InteractiveContext remains partial. Picking, broader display/erase/remove and selection
interaction, advanced attribute operations and its full dump are still open. ViewerSelector
traversal and Pick overloads are the recommended next dependency chain now that genuine
AIS downcasting and presentation lookup exist. [Next notes](next-context-notes.md) record
native ordering constraints and exact source locations.

[receipt.json](receipt.json) records counts and source hashes. [continuation.patch](continuation.patch)
contains all changes relative to the continuation parent and passes reverse checking;
generated inventory binaries are omitted and must be regenerated after integration.
The merged inventory assigns the two class records to TKVCAF through their first forward
declarations; the ledger retains this existing ownership while hashing the actual AIS
definition headers. This does not claim a TKVCAF behavior port. Historical checkpoints
remain unchanged.
