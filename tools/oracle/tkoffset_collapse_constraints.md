# BRepOffset::CollapseSingularities degenerated-vertex oracle

This probe compares the pinned native surface result for a nearly collapsed B-spline side
when the face has no degenerated constraint, one fitting vertex, two fitting vertices, or
one non-fitting vertex. It also checks the precision-only collapse branch. Each output row
records whether OCCT returned the input handle and all four poles, so the Rust test checks
both copy identity and every resulting pole.

The probe builds a degenerated edge with one shared vertex at both endpoints. The second
vertex occurrence uses reversed orientation, because OCCT's endpoint extraction considers
edge child orientation when deciding whether the vertices are the same.

Build from the repository root against the pinned OCCT modeling build:

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkoffset_collapse_constraints_probe.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKOffset -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_collapse_constraints_probe
/tmp/tkoffset_collapse_constraints_probe
```
