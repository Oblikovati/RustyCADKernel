# Martensite public ImageDumpOptions value evidence

Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 defines a public struct,
including eight mutable scalar/enum fields previously excluded as private state.
The task fragment records exact compiler declaration IDs and original header hash.
`LightName` and the struct remain partial; the existing default constructor keeps
its binding with corrected `Clone`, rather than `Copy`, notes.

The header-only native probe uses original pinned V3d/Graphic3d include directories
before installed dependencies. Fifteen rows repeat byte-for-byte across two runs;
its empty default LightName assertion also passes. The fixture covers defaults,
all named buffer/stereo choices, both booleans, signed assignments and independent
copied scalar fields. It does not prove general char-pointer or rendering behavior.

Rust replay and all 863 TKV3d tests pass. A final focused replay includes the added
empty-text assertion. Clippy, global formatter, explicit test-source formatter,
canonical discovery and its thirteen Python regressions pass. Every new function
has behavioral coverage; the small test/oracle modules meet the line limits.

Compiler proof reuses verified root-main pipeline 11455, captured at 336ecc788.
The owning production source `crates/tkv3d/src/v3d/enums.rs` is byte-identical in
main and this worktree, and all eleven preflight matches have that exact source
hash. Compressed raw TKV3d JSON, root capture manifest and bounded preflight are
retained. This is main compiler symbol proof plus independent new native/Rust
replay, not a new whole-worktree capture: this branch's new tests differ.

There are no renderer changes, GPU/performance claims, threshold changes or
whole-struct completion claims. Root owns canonical recapture after integration.
