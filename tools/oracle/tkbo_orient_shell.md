# Boolean shell orientation

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_orient_shell.cxx` records 192 OrientFacesOnShell observations: box,
cylinder, sphere, boundary-free face, empty shell and repeated opposite face
occurrences, each with eight face reversal masks and four shell orientations.
Replay: `cargo test -p rustycad-tkbo --test solid_orientation shell`.

The two source passes process exactly two unique adjacent faces first, then
non-paired adjacencies. Seam duplicates are removed by shape identity; degenerate
edges are skipped. Checks compare replacement identity, orientation, face count,
face identity/order and face orientation. Boundary-free faces vanish from the
output because OCCT populates it exclusively through its edge-to-face map.

These supplemental oracle fixtures leave general located and malformed shells
and full legacy-test parity unproven. This API remains partial.
