# Integrated explicit prism tolerance inspection

Integrated source: `a3245eace6611936fbeeae457d01bedfebe63ad0`.
Final reconciled candidate: `ed0475f8c`; original frozen source: `edf1de5ff`.
The two reconciliation folders retain registration changes and checks without
rewriting the original frozen evidence.

Root independently ran TKMath, TKBRep and TKBO all-target/all-feature suites with
Rust 1.99.0 and two build jobs: **6,138 passed, zero failed or ignored**, plus
**302 doctests**. Clippy with warnings denied and formatting passed. All 34
changed final candidate crate files match byte for byte. All tracked crate and
configuration hashes were unchanged across the matrix and additional replays.
Root independently verified all 53 original and reconciliation artifact hashes.

Two additional arithmetic replay processes match the 3,629-byte candidate
receipt, SHA-256
`a3bbc53e1b5be36f87331ff1319ce1e47b078447b694844d21f004abdaaaf8a7`.
Two additional frozen projection replay processes match the unchanged 89,921-byte
V1 baseline, SHA-256
`88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4`.
No original candidate test is missing. The 60 additional integrated tests are
listed in `integrated-replay.json` (complete measures/enclosures and embedding).

This inspection keeps raw stored metadata, compatibility accessor values,
explicit cap decisions and certified requested residual bounds separate. It
always retains unresolved placement semantics; it does not complete ordered
tolerance validation or publish a valid body. Its arithmetic replay is not a
complete body serializer. A nonzero requested-bound `Within` result is not an
attained maximum and is insufficient for the proposed exact-zero policy.

The follow-on policy must retain raw local vertex points and actual placements:
rounded BRepTool world points alone cannot prove a local allowance relation.
The independent rounding-trap regression and exact placement policy belong to
ADR0036, which is not implemented by this integration.

ThreadSanitizer, complete Kernel #10/publication, application adoption and owner
release gates remain open. No release-performance or full API/test parity claim
is made by these checks.
