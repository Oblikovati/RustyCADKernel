# Offset connectivity analysis oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkoffset_analyse.cxx` records a 1x2x3 box, reversed box, first box face,
a free line edge, empty compound, and three paired Bezier patches. Rust replays every record in
`crates/tkoffset/tests/analyse.rs`: completion, interval ranges/classifications,
ordered unique ancestor counts, all six concavity queries on vertices and faces,
tangent-edge counts, generated-state and Clear.

Uniform-offset incidence includes the 23-sample spline mixed-continuity check.
The paired-patch slopes 0, 1 and 3 yield Tangential, Concave and Mixed respectively.
Native C0Value() is a distance used with angle-like thresholds; this surprising
source behavior is deliberately preserved and covered by the Mixed fixture. Different face offsets and generated tangent faces,
progress cancellation, Explode, descendants and replacement/history queries are
not implemented. No legacy regression is mapped by these supplemental fixtures.
Native Perform retains old edge classifications across calls; only Clear removes
them. Rust preserves that source behavior; a repeated-input oracle remains needed.
