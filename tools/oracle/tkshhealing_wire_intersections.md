# Arbitrary wire edge intersections

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Eleven observations cover crossing lines, endpoint and single-endpoint contact,
overlapping segments, reversal, 3D representation separation, endpoint tolerance
filtering, missing pcurve, zero range, missing vertices and unloaded analyzer.
Sentinels verify that this overload retains outputs until both pcurves and their
ranges are accepted, then clears them before vertex access (including exceptions).
The spatial separation case exercises SameParameter-aware 3D evaluation.

Rows: `exception found DONE1 FAIL3 count2d count3d errors_count
[parameter1 parameter2 x y z error]*`. Exception 1 is Standard_NullObject.
All numeric fields are compared exactly, including parameters and spatial error.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_intersections.cxx`, output name `wire_intersections`,
and adding `-lTKGeomAlgo` after `-lTKShHealing`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_intersections
```

These observations do not establish full wire/toolkit parity.
