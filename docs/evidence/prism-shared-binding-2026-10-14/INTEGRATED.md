# Integrated shared prism binding

Candidate `f47001b3f` (frozen source `5685d1c0cd9140e67dd00ea05b5cf01076784d3c`)
was integrated at `83eccdbcc63c6a672df5e0c5facbdcca79df2c37` without conflict.
All 11 added or modified candidate crate files match exactly. All four old
measurement binding/chart/history/replay paths are absent. The updated integration
runner checks removed paths as well as file contents; its complete source is
retained in `integrated-runner.py.gz`.

Rust 1.99.0 passed **1,149 TKBO tests** and **35 doctests**, with zero failures
or ignored tests. The all-target run includes the two existing `edge_private_probe`
example tests in addition to the candidate's 1,147 unit/integration tests.
All-target/all-feature Clippy with warnings denied and workspace formatting
passed. All tracked crate/config input hashes were unchanged across the run.
Candidate evidence archive checksums also passed independent verification.

The shared capture/rebind implementation replaces the four former private
measurement modules while preserving their arithmetic, cancellation sequence,
refusal variants and existing owned fixtures. The new baseline tests retain
current cubic/seam observations and the genuine stationary-quartic refusal.
The ADR0029 embedding theorem is still a design; this batch implements no new
embedding theorem, ordered validation completion or successful body result.

Commands, counts, logs and source/removal comparisons are in `integrated-main.json`
and the other `integrated-*` artifacts. This is local integration evidence, with
no application, release, performance, sanitizer-completion or full API/test parity
claim. Earlier sanitizer startup limitations remain explicitly incomplete.
