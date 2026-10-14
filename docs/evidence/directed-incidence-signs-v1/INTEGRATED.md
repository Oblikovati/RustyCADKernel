# Integrated directed-incidence verification

Local main revision `0e1f2ef8f025d83f2ffcb7178a9820d06c0f1eed` integrates
candidate `63f75308c`. The coordinator independently ran the combined TKMath
and TKBO all-target/all-feature suite with Rust 1.99.0 and two build jobs:

- 4,169 tests and 86 doctests passed; zero failures or ignored tests.
- All-target/all-feature Clippy with warnings denied passed.
- Workspace formatting passed.
- Every candidate crate file matches the integrated bytes. All tracked crate,
  workspace manifest, toolchain and Cargo configuration inputs were unchanged
  between the before-build and after-check snapshots.

[integrated-main.json](integrated-main.json) records commands, platform, source
revision, comparison results and compressed-log hashes. The compressed runner
and complete source bookends accompany the receipt. Existing benchmark workload
gates ran as test targets; this is not release performance evidence.

The operation remains a conditional incidence certificate, with no nearest-hit
ordering or complete trim/extension operation claim. ThreadSanitizer, application,
release-package, cross-platform and full API/test parity acceptance remain open.
