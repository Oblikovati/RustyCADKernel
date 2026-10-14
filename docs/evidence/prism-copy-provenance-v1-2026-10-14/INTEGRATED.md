# Integrated immutable prism copy provenance

Integrated source: `0ba44a8fd0de5ee5d0ee247cac7374cd2e7c1c0b`.
Reconciled candidate: `b05bc6255`; original behavior: `cc3efbf37`.
Original and reconciled evidence remain separate and retain their cited revisions.

Root ran the complete affected TKBO all-target/all-feature suite with Rust 1.99.0
and two build jobs: **1,297 passed, zero failed or ignored**, plus **35 doctests**.
Clippy with warnings denied and formatting passed. All 19 changed candidate crate
files match byte for byte. All tracked crate/configuration hashes were unchanged
across the matrix and four additional replay processes. Root independently
verified all 51 original/reconciled artifact hashes before integration.

This integration changes only TKBO. It does not repeat the unchanged dependency
suites: the prior tolerance integration at `a3245eace` recorded 6,138 tests and
302 doctests across TKMath, TKBRep and TKBO. The current root count is explicitly
TKBO-only. No original candidate TKBO test is missing; 86 additional integrated
test names are recorded in `integrated-replay.json`.

Two lineage processes match the candidate's 167,821-byte receipt, SHA-256
`491a1a0a7d1a68505a31dcbc3ea52737d19542d44b3a9e5fb93a072614e397ec`.
Two frozen projection processes match the unchanged 89,921-byte V1 receipt,
SHA-256 `88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4`.
The replay tests also exercise repeated operations and concurrent threads.

The inspection retains actual selected-copy observations, directed contexts,
all parent multiplicities, sweep images and intermediate retirement. It checks
these again after the final callback and owns its immutable replay operands.
It does not prove exhaustive upstream partition dispositions, empty sweep query
coverage, persistent application naming, original deletion or whole-body validity.
Those obligations remain explicit for the bounded history composition follow-on.

ThreadSanitizer acceptance, full Kernel #10 publication and general profile
coverage, application adoption and owner release gates remain open. These checks
do not establish full API/test parity or controlled release performance.
