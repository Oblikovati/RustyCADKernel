# Martensite AIS view-controller event integration evidence

This slice completes 66 public controller methods: input queue access, public
input policies, selection gestures, zoom/wheel coalescing, touch Z rotation,
navigation transitions and the previous-highlight sentinel. The controller
record and constructor remain partial; unfinished dispatch, animation, input
listener and XR methods stay open. No API exclusion or denominator change was
introduced to claim completion of the class.

The source reference is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The local pinned native build generated 80 committed snapshots. Two independent
runs reproduced the fixture byte-for-byte. Header, generator and fixture hashes
are recorded beside the asset. Rust tests never link or execute OCCT.

The 33 new regressions replay all snapshots. They cover every public input policy
getter/setter, negative unclamped scalar values, isolated UI/render buffers,
picking accumulation, empty/one/two/three-point sequence selection, owned point
snapshots, rectangle/polygon/pick transitions and retained apply flags, duplicate
polygon vertices and replacement, adjacent-only zoom coalescing, cancellation
to a retained zero delta, first-modifier retention, wheel scaling, allow-zoom
policy independence during queuing, gated touch rotation, acknowledged/restarted
angles, same-mode navigation cancellation and previous-highlight reset.

All 515 TKV3d tests pass on both the feature checkout and integrated main.
Clippy with warnings denied, Rust formatting, canonical toolkit discovery and
12 suite-generator regressions pass. The renderer is unchanged; its existing
paired-device capture/performance gates remain in CI. The compressed logs retain
the commands' results, and the integration receipt records the compiler-backed
inventory, source inputs and cleanup audit.

The feature commit is integrated directly into main. Its clean completed worktree
and branch were deleted after verifying ancestry, source equality and no active
process references. Deletion reclaimed 2.71 GiB; no archive was made.

See [implementation notes](../../../docs/ais-view-controller.md). Continue with
keyboard navigation and the remaining controller dispatch using the existing
window-input listener base and deterministic timer injection; do not replace
these unfinished methods with placeholders or remove them from inventory scope.

The refreshed inventory has zero unresolved ledger rows. Port-scope done APIs
rise from 52,195 to 52,261; open APIs drop from 22,871 to 22,803, with partial APIs
rising from 975 to 977. TKV3d has 2,640 done declarations out of 5,172 (51.0%),
2,528 open and four partial. Full port completion remains unproven, including the
explicit compiler configuration/identity/mapping proof gaps.
