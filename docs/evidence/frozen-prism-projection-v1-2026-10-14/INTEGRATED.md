# Frozen prism projection integration

Integrated source: `bdb88a54723bf14dc4be6be04a326043ef16a564`.
Candidate source: `2e0cf4817da4f47f8f0b3e8bba28dc8a7425c6e1`;
candidate evidence tip: `35d752db27bb2eec8db55e190de9ca78b4366e57`.

The coordinator reran TKMath, TKBRep and TKBO all-target, all-feature tests:
**6,039 passed, zero failed or ignored**. All **302 doctests**, Clippy with
warnings denied, and workspace formatting passed on Rust 1.99.0 with two build
jobs. All 35 changed candidate crate files are byte-exact on integrated main.
Tracked crate/configuration input hashes were unchanged across the matrix.
The candidate's 30 retained artifact hashes were independently verified.

The nine additional tests relative to the candidate's 6,030 come from the
already integrated shared binding: four embedding baselines, four typed failure
conversion tests, and one owned-operand Send/Sync assertion. No candidate test
name is absent from the integration log.

Two further integrated processes each executed the independent-operation/thread
replay. Both produced the same 89,921 bytes as the candidate, SHA-256
`88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4`.
Commands, compressed logs and output hashes are in `integrated-replay.json`.
The complete matrix, candidate comparisons and source bookends are in
`integrated-main.json` and its referenced artifacts.

This is an owned diagnostic capture, not a successful solid publication.
All four ordered body validation stages remain incomplete. The ordinary-host
ThreadSanitizer startup control failed before its marker; the projection target
was not executed under TSan. Application, release, performance and full API/test
parity gates remain open. No host protection or sanitizer bypass was used.
