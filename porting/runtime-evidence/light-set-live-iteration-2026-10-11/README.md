# Martensite light-set live iteration local receipt

Base: `54382fc12cd0a93a06db00842661c16ea1e5d626`.
Branch: `porting/martensite-light-set-live-iteration`.
OCCT oracle pin: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

This bounded correction replaces snapshot membership in the LightSet iterator
with shared live indexed storage and typed native exhaustion errors. It also
fixes the `i32::MIN` bounds overflow and preserves swap-last removal, cached
state timing and revision coalescing. Cache helpers were split without changing
native accumulation semantics. Public usage and limitations are recorded in
`docs/graphic3d-light-set-live.md`.

`checks.json` records commands, counts, exact log hashes and limitations.
TKService all-targets: 89 library +160 toolkit tests pass, including four new
regressions and 576 committed CPU-native transitions. Clippy, global formatting,
canonical discovery and its 13 tooling tests pass. The native output was repeated
byte-identically. Provenance retains generated include-wrapper and resolved
pinned-header identities separately, together with native library/binary hashes.

The unique task fragment pins 25 corrected existing operations and leaves both
class records partial. It claims no private helper or new getter progress.
`rust-selector-audit.json` derives all 27 bindings from a fresh single-crate
rustdoc capture; its two Default constructor bindings retain exact concrete impl
contexts/digests. This is a local selector check, not a replacement for the root
agent's canonical full-workspace inventory publication. One pre-existing
Graphic3dStructureMembers doc-link warning appeared during capture; no LightSet
warning or failing check remains.

No renderer submissions or GPU workloads changed. Linux CPU parity is verified;
Windows/macOS checks are deferred. The coordinating agent must integrate the
commit into main, run combined checks, and physically delete this completed
checkout and merged branch after auditing that no process needs its artifacts.
