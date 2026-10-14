# Exact sketch edit comparisons

## Contract and source

ADR 0013 adds `compare_squared_distances_2d` and
`compare_line_projections_2d` to TKMath's exact predicate boundary.
The implementation reuses the binary64 integer representation from ADR 0003.
It adds no dependency, rounded normalization, metric epsilon, or retry path.
Non-finite inputs retain the existing typed coordinate error, in argument order.
A zero projection direction produces equality; callers still validate lines.

The base is kernel `f1afb9ef0`. The Go reference is
`aa8297d1b0ac33c024f19e54821536d76e8a447a`. Application epic
[#3563](https://github.com/Oblikovati/Oblikovati/issues/3563) owns the R3 edit
port and deletion of Go endpoint selection and extension ordering.
These functions only order supplied coordinates. They do not certify the
construction error or completeness of intersection roots.

The shared integer dot product replaces the expression previously inside
`roots/bezier_line/projection.rs`. Its operands, multiplication order, and
addition expression are identical. Integer arithmetic is exact; extraction
therefore preserves each existing value, including sign and zero. The existing
Bezier root tests run in the complete TKMath suite. Mapped OCCT metric APIs
are unchanged. Net additions to recognizers, tolerance constants, fallback
sites, and type assertions are all zero.

## Executed checks

Local Linux checks on 2026-10-13 used the locked dependencies and Rust 1.99.0:

- `cargo test --locked -p rustycad-tkmath --all-features`: 14 unit tests,
  2,846 integration tests, and 46 documentation tests passed, with none ignored.
- Clippy over all TKMath targets and features passed with warnings denied.
- TKMath documentation built with warnings denied.
- Workspace Rust formatting and toolkit test discovery passed.
- The eight new integration tests passed against 296 independent Python
  `Fraction` cases, operand reversals, every finite binary64 exponent field,
  signed zeros, exact ties, and all non-finite argument positions.
- Eight ordinary cases matched native OCCT 8.0.0 metric comparisons. Extreme
  and near-tie expectations come from exact rational arithmetic, not OCCT's
  rounded distance and dot product. Both fixtures reproduced byte for byte.
- Focused LLVM coverage measured 32/32 lines, 64/64 regions, and 4/4 functions
  in `predicates/compare.rs`. This is new-module coverage, not toolkit coverage.
- The predicate-directory duplication scan found zero clones across 25 Rust
  files and 2,358 scanned lines, using five-line and 50-token thresholds.

One debug run of the same eight accepted tests took 0.02 seconds wall time
and 16,188 KiB peak RSS. This is a local diagnostic measurement, not an
optimized or cross-platform performance guarantee. Fixture replay commands
are in the [corpus README](../crates/tkmath/tests/exact_comparisons/README.md).

The test helpers were then split for readability. The independent Fraction
fixture still reproduced byte for byte, and all eight focused tests passed
again. The final all-target TKMath Clippy check, formatting, test discovery,
Markdown checks, and new-source SPDX checks also passed. Kernel main advanced
only by cleanup evidence commit `8062f0f23`; that commit was merged before
local integration. No production code changed after the full TKMath run.

## Remaining acceptance

The full renderer and owner release gates were not run for this predicate
change. No release, push, or application source override is included.
The app remains on v0.2.5. A later owner release and exact app intake must
precede production use. Line extension still needs certified incidence
selection, atomic endpoint replacement, identity preservation, undo, persistence,
and native edit acceptance. These tests do not complete that operation.
