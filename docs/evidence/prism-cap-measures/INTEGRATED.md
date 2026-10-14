# Integrated cap and signed-sweep measurements

Frozen source `9584ca4388245ed6ac111aec32642c6e797752ff` and candidate evidence
`2217eb979e497b8ffd668bf7ad45eb8d42d2c046` were integrated at
`ab5fe6414104a420718dad11e03f5a729798791e`. All 30 changed candidate crate files
match that frozen source exactly. The whole tracked source-input bookends match.

Rust 1.99.0 passed **4,321 TKMath and TKBO tests** and **88 doctests**, with
zero failures or ignored tests. All-target/all-feature Clippy with warnings
denied and workspace formatting passed. The three additional tests relative to
the candidate's 4,318 are the previously integrated cumulative-pcurve prism graph
regressions. Candidate archive checksums also passed independent verification.
Commands, compressed logs, source hashes and the runner are retained in
`integrated-main.json` and the `integrated-*` artifacts.

The only merge conflict was historical evidence text: the corrected regularity
candidate count of 18 was retained instead of 19. There was no production merge
conflict. The candidate's original base `a42ab0646` remains recorded unchanged.

This adds conditional world-plane area bounds and binds the two cap measurements
and signed sweep integral to freshly checked source, history, charts and trims.
Side and total area remain pending. No successful body, full ordered validation,
application acceptance, release, performance result or API/test parity is claimed.
The candidate's ordinary-host ThreadSanitizer execution crashed before test
output; sanitizer acceptance remains incomplete. No host protection was changed.
