# Area-based face subdivision oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Compile `tkshhealing_face_divide_area.cxx` against the pinned native build with
`-std=c++17 -O2`, its include and library directories, and libraries
`TKShHealing TKTopAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel`.
The committed expected output is native output. Reproduce Rust comparisons with:

```sh
cargo test -p rustycad-tkshhealing --test shape_upgrade_face_divide_area
```

Eight summary rows cover unchanged area, maximum-area subdivision, deriving the
maximum from a requested count, fixed U/V counts, null and incompatible surface
criteria, zero parts (native infinite maximum), and recursive spherical splitting.
The sphere additionally compares all twenty result faces in native traversal order,
including four UV bounds and integrated area (absolute tolerance 1e-12). Summary
floating values use absolute tolerance 1e-14; discrete fields compare exactly.

The sphere exposed reversed side codes in ComposeShell's external split-edge
branch. The split branch and unsplit endpoint branch intentionally use opposite
ternaries in upstream; both are now retained, with a focused regression.

These are additional oracle tests, not additional mapped legacy declarations.
Perform remains partial because configured policies and shell composition still
have documented limits. Positive U/V integer product overflow raises a checked
Rust error; native signed overflow has no defined parity claim. Invalid/nonfinite
part conversion and adversarial recursion require further evidence. No whole-class
or toolkit completion claim is made.
