# Solid box offset and vertex closure

Pinned OCCT offsets a `1 x 2 x 3` box as a solid. The Rust builder now produces the
same 26-face, 96-edge topology: six shifted planes, twelve edge-join cylinders, and
eight spherical vertex closures. Whole-solid and per-face `BRepCheck_Analyzer`
validation pass, and source face/edge/vertex history counts match this fixture.

Surface area still differs slightly. OCCT reports `43.991148576` at the fixture's
stable nine-digit precision; Rust reports `43.99114858034269`, a `3.94e-9` excess
against the full-precision native run. The eight Rust spherical faces are each about
`4.9e-10` larger than their native counterparts; plane and cylinder areas agree at
the printed precision. The Rust test uses `1e-8` only to keep this topology fixture
stable. Numerical parity for spherical pcurves remains unresolved, so this result is
not evidence of full `BRepOffset_MakeOffset` parity.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" tools/oracle/tkoffset_standard_box_solid.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_box_solid
/tmp/tkoffset_box_solid
```

The captured output is `tkoffset_standard_box_solid.expected`.
