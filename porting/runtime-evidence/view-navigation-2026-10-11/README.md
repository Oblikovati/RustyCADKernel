# Martensite view navigation integration evidence

The public APIs in `AIS_MouseGesture.hxx`, `AIS_NavigationMode.hxx`,
`AIS_RotationMode.hxx` and `AIS_WalkDelta.hxx` have 58 compiler-bound Rust
counterparts. The two anonymous range enums have named Rust types. Their ledger
identities use source-relative header paths so another checkout can resolve them.
No scope exclusions or denominator changes were introduced.

The pinned reference is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The local header-only oracle reproduces all 57 committed snapshots byte-for-byte,
without linking the native kernel. Nine Rust regressions replay enum/bound values,
defaults, exact DBL_MIN boundaries, signed zeros, subnormals, NaN/infinities,
held-key/stance combinations, independent copies and all six axes. The Rust tests
require no OCCT headers, runtime library or generator invocation.

All 482 TKV3d CPU tests pass. Clippy with warnings denied, Rust formatting,
canonical toolkit suite discovery and 49 inventory/suite tooling tests pass.
The existing 12 anonymous-identity regressions also pass after correcting the
ledger paths. The gzipped logs preserve the executed checks.

The feature commit is integrated into main. The completed worktree and branch
were deleted after checking source equality, commit ancestry, a clean checkout
and absence of active process references. Deletion reclaimed approximately 2.8
GiB; no archive was created. The integration receipt records the fresh inventory
counts and source proof. Full port completion remains unproven because remaining
APIs and compiler configuration/identity/mapping proof gaps are still open.

See [implementation notes](../../../docs/ais-view-navigation.md) for public usage
and local fixture regeneration. Next, port AIS view-controller event dispatch,
including selection accumulation, zoom coalescing, Z rotation and navigation-mode
transitions using the now-ported input records and navigation value types. Keep
unimplemented view-controller methods open in the inventory.
