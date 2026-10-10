# Viewer selector geometry continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main because session filesystem policy makes Git metadata read-only. Preserve
this checkout and all original Claude worktrees. Twenty-five committed continuation
commits remain outside main; the original dirty TKernel linked-slot refactor is untouched.

The [fresh inventory](METRICS.md) accounts for **51,318/76,041 defined APIs
(67.5%)** in the unchanged 56-toolkit scope. Two public/protected mappings
are newly implemented: updateZLayers and ClearSensitive. Done includes prior exclusions,
not complete behavior parity. Counts describe the continuation, not main. Every new
mapping resolves uniquely; unresolved ledger rows remain zero.

The private picked-point/depth-tolerance and per-entity frustum helpers are translated
and compared against actual linked OCCT private member functions. Normal calculations
preserve float rounding, initial-location composition uses the criterion entity, and
tolerance sensitivity uses the separate argument entity. Only scaled volumes without
transformation are cached; object clipping is copied after cache lookup without changing
cached state. Layer ordering follows depth-clear/depth-write transitions. Debug cleanup
follows live sequence links, erases/clears/removes structures without callback locks,
clears the sequence, then updates a non-null view. Full traversal and AIS presentation
lookup remain unimplemented; ViewerSelector remains partial.

Checks passed: **243 TKV3d Rust tests**, **130,729 native comparisons**, one native and
one Rust mapped legacy case, Clippy with warnings denied, seven tooling tests, formatting
and diff checks. Ten direct tests and 2,656 numeric cases are new: 720 point cases,
902 frustum cases, ten debug-cleanup cases and 1,024 layer patterns. Singular transforms
preserve native nonfinite point results or construction failures, depending on the path.
Failed frustum construction preserves the output and does not populate the cache.

Native rectangular scaling/transformation with a cleared camera crashes in
cacheVertexProjections at Camera()->IsOrthographic(). [The input](native-undefined-sequence.txt)
and [stack](native-crash-stack.log.gz) preserve this finding. Rebuild cases retain a
camera; null-camera point updates and unchanged rectangles remain compared. No native
undefined-memory parity claim is made. The fixture calls actual private methods via
standard C++ explicit-instantiation member-pointer access; it copies no native bodies.

[receipt.json](receipt.json) records counts and source hashes. [continuation.patch](continuation.patch)
contains all changes relative to the continuation parent and passes reverse checking;
generated inventory binaries are omitted and must be regenerated after integration.
[next-context-notes.md](next-context-notes.md) records the remaining context/object and
traversal dependencies. Historical checkpoints remain unchanged.
