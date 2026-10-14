# Combined main verification

Candidate `39af64b98` (frozen source `7a2b9e388`) was integrated at
`99849a27da57332460482fe3cd3c29df7b487c7d`, with current side-integral and
regularity dependencies. All 18 candidate crate files match the tested
candidate exactly.

Rust 1.99.0 passed **2,786 TKBRep and TKBO tests** and **249 doctests**, with
zero failed or ignored tests. All-target/all-feature Clippy with warnings denied
and workspace formatting passed. All tracked crate and workspace build inputs
remained unchanged throughout the combined run.

`integrated-main.json` records commands, counts, platform, source hashes and
compressed-log hashes. The actual runner and complete source bookends are
retained. The candidate's original finalizer failure and explicit recovery remain
unchanged alongside these independently executed integrated checks.

V2 selects cumulative occurrence placement for newly extracted pcurves. Existing
V1 behavior remains default and selectable; existing graph definitions are not
repaired by append. Rebuilding and auditing is required for migration. This
prerequisite does not certify complete body validity, immutable graph ownership,
full API/test parity, ThreadSanitizer, application or release performance.
