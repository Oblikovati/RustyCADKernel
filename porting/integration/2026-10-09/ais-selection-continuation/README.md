# AIS selection continuation

Validated source: `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Changes are uncommitted and not integrated
into main because session filesystem policy makes Git metadata read-only. Preserve
this checkout and all original Claude worktrees; the original dirty TKernel linked-slot
refactor on `worktree-agent-aeed1901b085eca95` remains untouched and unvalidated.
Twenty-five committed continuation commits also remain outside main.

The [fresh inventory](METRICS.md) accounts for **51,316/76,041 defined APIs
(67.5%)** in the unchanged 56-toolkit scope, adding 39 implemented
mappings. Done includes prior exclusions, not complete behavior parity. Counts describe
the continuation, not main. Every new mapping resolves uniquely; unresolved ledger rows
remain zero. ViewerSelector picking/traversal and AIS context remain incomplete.

AIS_Selection now provides its complete public/protected protocol: virtual mutation,
owner membership, ordered list, retained public cursor, native map sizing and borrowed
actual list/value access. Forced highlighting observes the advanced cursor; ReplaceExtra
uses count and membership even for duplicate picks. Filters can fail or reenter without
holding selection state locks. AIS_GlobalStatus preserves independent display/highlight/
sub-intensity state, unique ordered modes and borrowed drawer identity. AIS_SelectStatus
and the exact StdSelect_ViewerSelector3d typedef are ported. The alias does not imply
completion of its partial underlying viewer-selector class.

SelectMgrFilter::is_ok now returns Result<bool, Failure>, preserving native exceptions
through selection and ordered AND/OR composition. Disabled objects short-circuit callbacks.
Safe errors cover native-invalid null appendOwner filters and invalid Value cursors;
these cases are tested directly and not assigned native undefined-memory parity claims.

Checks passed: **233 TKV3d Rust tests**, **128,073 native comparisons**, one native and
one Rust mapped legacy case, Clippy with warnings denied, seven tooling tests, formatting
and diff checks. Fourteen direct tests and 357 numeric cases are new: 205 selection
traces, 80 status traces and 72 filter composition cases. Exceptions, reentrant callbacks,
mutation during iteration, borrowed identity and virtual dispatch are covered.

[receipt.json](receipt.json) records counts and hashes. [continuation.patch](continuation.patch)
contains all source changes relative to the continuation parent and passes reverse
checking; generated inventory binaries are omitted and must be regenerated after
integration. Historical evidence remains unchanged. [next-context-notes.md](next-context-notes.md)
records remaining context/object dependencies and independent viewer geometry work.
