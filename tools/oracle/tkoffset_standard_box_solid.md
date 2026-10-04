# Solid box offset and vertex closure

Pinned OCCT offsets a `1 x 2 x 3` box as a solid. The Rust builder now produces the
same 26-face, 96-edge topology: six shifted planes, twelve edge-join cylinders, and
eight spherical vertex closures. Whole-solid and per-face `BRepCheck_Analyzer`
validation pass, and source face/edge/vertex history counts match this fixture.

The former `3.94e-9` excess came from approximating elementary pipe connector
isocurves as 3D B-splines before projecting them onto the vertex spheres. The native
connector probe in `tkoffset_sphere_connector_curves.md` established the exact circle
and p-curve types. The Rust builder now stores those elementary isos exactly, and its
whole-solid area is within `1e-12` of the native full-precision fixture. This resolves
the area discrepancy for this box input; it does not establish full
`BRepOffset_MakeOffset` parity.

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
