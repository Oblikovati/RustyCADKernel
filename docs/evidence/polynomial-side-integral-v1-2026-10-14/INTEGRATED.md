# Combined main verification

The conditional polynomial side-integral candidate `52eb7bda9` (source
`e60e9b30b`) was integrated at `8980e4929407772a7a859c47287ec37208c5b643`.
The merge also includes documentation-only commit `970ea1bce`, which qualifies
owning issue references without changing checked source, build configuration or fixtures.

On integrated main, Rust 1.99.0 passed **4,238 TKMath and TKBO tests** and
**87 doctests**, with zero failed or ignored tests. All-target/all-feature Clippy
with warnings denied and workspace formatting passed. Every candidate crate file
matches its tested candidate exactly. All tracked crate and workspace build inputs
are unchanged across this combined run.

`integrated-main.json` records commands, counts, platform and SHA256 links to
compressed logs and complete source bookends. `integrated-runner.py.gz` retains
the actual runner. These checks combine the new scalar operation with the current
region-certificate and directed-incidence implementations.

This is a conditional scalar integral with certified bounds. It does not prove
face correspondence, regularity, body validity, full API/test parity, application
acceptance, ThreadSanitizer completion or release performance.
