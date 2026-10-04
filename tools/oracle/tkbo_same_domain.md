# BOPDS same-domain chain regression

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`BOPDS_DS.cxx` AddShapeSD, HasShapeSD and GetSameDomainIndex.

The 18-row fixture checks absent mappings (including preservation of the output
sentinel), ignored self-bindings, four-hop lookup, replacement of an intermediate
binding, and negative integer keys. Rust also checks that resolved lookup does
not compress or otherwise change the direct bindings exposed by ShapesSD.

Cycles are outside this fixture: the native implementation does not terminate
on cyclic mappings. No cycle recovery behavior is claimed. This supplemental
regression does not count as a ported legacy test declaration.

Build the C++ probe against the pinned modeling build's include/opencascade and
lin64/gcc/libi, linking TKBO and TKernel, and compare stdout to the expected file.
Run the Rust replay with `cargo test -p rustycad-tkbo --test bop_ds_same_domain`.
