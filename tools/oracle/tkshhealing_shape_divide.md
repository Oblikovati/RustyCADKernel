# Whole-shape division oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkshhealing_shape_divide.cxx` with `-std=c++17 -O2`, the pinned native
include/library directories, and libraries `TKShHealing TKTopAlgo TKBRep
TKGeomBase TKG3d TKG2d TKMath TKernel`. Its output is committed as `.expected`.

```sh
cargo test -p rustycad-tkshhealing --test shape_upgrade_shape_divide
```

Fifteen cases cover null-source failure, unchanged faces and free edges, area-driven
face splitting, reversed assemblies with shared located faces, preserved versus
replaced contexts, context removal without DONE status, caught face policy failure,
and free-wire/free-edge policies reporting both DONE and FAIL. Output compares
result identity, occurrence counts, context/source identity, five status flags,
message count, reversed face occurrences and summed integer Z placements.
Named policy fixtures implement actual native virtual extension points in both
languages. Additional Rust tests check raw settings and retained result state.

Native's null face-tool case dereferences a null handle in free-boundary processing.
Rust returns `Standard_NullObject`; this is an explicit unapproved exceptional-domain
difference, not successful parity evidence. Signal-to-exception conversion and other
child-tool limitations remain outside these cases. Perform remains partial. These
additional oracle cases do not increase the legacy declaration mapping count.
