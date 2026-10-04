# Offset connectivity analysis oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkoffset_analyse.cxx` records a 1x2x3 box, reversed box, first box face,
a free line edge, and empty compound. Rust replays every record in
`crates/tkoffset/tests/analyse.rs`: completion, interval ranges/classifications,
ordered unique ancestor counts, all six concavity queries on vertices and faces,
tangent-edge counts, generated-state and Clear.

Initial scope is uniform-offset incidence and non-spline classification.
Paired spline surfaces return an explicit NotImplemented error pending the mixed
continuity translation. Different face offsets and generated tangent faces,
progress cancellation, Explode, descendants and replacement/history queries are
not implemented. No legacy regression is mapped by these supplemental fixtures.
Native Perform retains old edge classifications across calls; only Clear removes
them. Rust preserves that source behavior; a repeated-input oracle remains needed.
