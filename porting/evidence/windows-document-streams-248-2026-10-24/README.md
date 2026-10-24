# Windows destination-stream refusal (#248)

Native Windows x64 execution reproduced destination metadata loss in release
candidate `2254483e14b35f82a838a316ef5121d6059de658` and verified the correction
at `f58319b3547a2d421e2d54607f9eb4556b0bc3da`.

## Observed failure and correction

[Run 40075229497](https://github.com/Oblikovati/RustyCADKernel/actions/runs/40075229497)
executed three save paths twice on NTFS with Rust 1.99.0. The actual
`File::create` control wrote replacement bytes while preserving the seeded
`Zone.Identifier` stream. The separate `fs::rename` control and the candidate's
access facade followed by its exact sync/rename publication sequence both wrote
replacement bytes and lost that stream. The red-capable probe failed with exit
101 after both repetitions. This is a native observation, not a whole historical
SDK-save comparison or a cross-compilation inference.

The correction enumerates destination streams before changing staged-file
access metadata. Only the unnamed `::$DATA` byte stream is accepted. Other
streams produce `PermissionDenied`; enumeration failures retain their native
codes, and only enumeration EOF establishes completion. The search handle is
closed on success and refusal. The document writer's existing error conversion
returns `DocumentStore_PermissionDenied`, and its staging owner removes the
private directory after refusal. No destination stream is silently discarded.

[Run 40076129216](https://github.com/Oblikovati/RustyCADKernel/actions/runs/40076129216)
passed. Both repetitions refused publication, retaining original bytes and
stream contents. Twelve native Windows foundation tests passed, including
ordinary existing/new-file saves, restrictive and read-only writer policy,
Zone.Identifier and arbitrary named-stream refusal, unchanged destination/stage
policy snapshots, enumeration failures and malformed-name rejection. The native
document-writer test also passed, checking the typed error and staging cleanup.

## Evidence identity

The fixed diagnostic checked out the exact candidate and applied the retained
patch. Before compilation, every changed file's SHA256 matched the frozen fix
manifest. Its SDK Git HEAD remained the candidate base; the tested working tree
was base plus that verified patch. The standalone probe reused the baseline
resolved lockfile with `--locked`; the native product tests used the SDK
lockfile with `--locked`. Both lockfiles and the compiler identity are retained.

The initial Windows checkout converted probe line endings to CRLF. Its recorded
source hash equals the retained probe after only LF-to-CRLF conversion. The
fixed transport materialized patch inputs and affected base files from exact Git
blobs. An earlier attempt correctly failed the patch SHA gate before compilation;
its raw log is retained. Superseded run identities are recorded in the receipt.
The source workflow retains its existing runner, pinned actions and read-only
permissions. The product workflow was unchanged; only the isolated diagnostic
branch was pushed.

The archives contain raw workflow output, observations, executable/source hashes,
filesystem information, compiler revisions and resolved lockfiles. The receipt
binds the runs, tested source, observations and archive hashes. Windows-target
library/test checking and strict Clippy also passed locally. Three documented
unsafe blocks call the Windows stream-search APIs; final inventory admission
remains a release gate.

This focused proof does not qualify the final merged release SHA or replace its
required CI and four-platform release matrix. The product fix remains pending
coordinator integration through the Martensite PR workflow.

Microsoft documents named-stream preservation and partial-failure states for
[ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew),
so it was not substituted blindly for the existing publication operation.
[FindFirstStreamW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirststreamw)
and [FindNextStreamW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findnextstreamw)
define the enumeration/EOF contract used by the guard.
