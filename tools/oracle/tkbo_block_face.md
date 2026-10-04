# Boolean edge-block face membership

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_block_face.cxx` records 90 IsBlockInOnFace cases across five surface
fixtures, full/interior/outside ranges, three spatial translations, and two
edge tolerances. Replay:
`cargo test -p rustycad-tkbo --test make_pcurve block_face`.

Checks include the result and final cached surface projector state, distance,
and UV. This preserves the 0.0075 endpoint shrink, endpoint projection checks
without distance rejection, and the asymmetric interior sample's distance test.
Short-circuiting leaves the native final projector state intact.

This supplemental differential evidence does not establish arbitrary surface
behavior or full legacy-test parity. The API mapping remains partial.
