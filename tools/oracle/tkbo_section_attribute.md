# Section intersection flags

OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`: inline
`BOPAlgo_SectionAttribute.hxx` and `BOPAlgo_PaveFiller::SetSectionAttribute`.
The 60-row native fixture contains the default, all eight constructor states, all
48 single-flag Boolean transitions, and three PaveFiller configuration snapshots.
The last snapshots check default flags, value assignment independent of later
source mutation, and retention through Clear.

Build `tkbo_section_attribute.cxx` against the pinned modeling installation with
TKBO, TKPrim, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath
and TKernel, then compare stdout with `tkbo_section_attribute.expected`.

Run `cargo test -p rustycad-tkbo --lib --test bop_algo_section_attribute` to replay
the fixture. The public flags use Rust Boolean fields; Rust Copy expresses the
native value-copy contract. These supplemental tests add no legacy declarations.
PaveFiller configuration storage is implemented; full face/face intersection
consumption and the aggregate Boolean pipeline remain separate obligations.
