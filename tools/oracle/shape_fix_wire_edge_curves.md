# ShapeFix_Wire::FixEdgeCurves oracle

The pinned OCCT probe records the unloaded no-op, explicit add-p-curve pass,
the default mode sequence, and removal of an incomplete edge without a face.
It checks aggregate status bits, edge count, and whether the p-curve was attached.
The explicit add-p-curve and incomplete-edge rows disable the unrelated modes.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/shape_fix_wire_edge_curves.cxx \
  -L"$build/lin64/gcc/libi" -Wl,-rpath,"$build/lin64/gcc/libi" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/shape_fix_wire_edge_curves
/tmp/shape_fix_wire_edge_curves
```

Expected output is pinned in `shape_fix_wire_edge_curves.expected`.

The `sphere-pole` case uses an explicit reshaper context and a circular 3D edge
whose interior crosses the spherical north pole. OCCT splits it at the projected
singularity; the recorded edge ranges and DONE3/FAIL3 bits are checked in Rust.

Additional rows bind the ten mode defaults, the no-op behavior of p-curve and
3D-curve removal when their endpoint checks pass, and the plane-projection
no-op. The sphere-pole fixture also checks that both the edge and p-curve
ranges are restricted to each resulting segment.
