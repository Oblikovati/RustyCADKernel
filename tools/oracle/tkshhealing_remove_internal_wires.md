# ShapeUpgrade_RemoveInternalWires oracle

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`. The probe builds a real
planar face with one square internal wire and records full-scan removal, face-removal mode,
area threshold (including NaN comparison), selected-wire/foreign-face processing, one and
multiple adjacent-face removal, persistent reshape context across `Init`, status flags,
removed-shape counts, and result topology counts. `ShapeAnalysis::ContourArea` supplies the
same sampled 3D contour area used by the native algorithm.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_remove_internal_wires.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_remove_internal_wires
/tmp/tkshhealing_remove_internal_wires
cargo test -p rustycad-tkshhealing --test shape_upgrade_remove_internal_wires
```

Additional Rust regressions cover getter reentry from a named curve observer while a
second hole is sampled, incremental visibility of the first removed wire, retained
sequence identity across the protected `Clear`, and the native removed-face iterator's
advance after a successful match. Geometry/context calls execute outside the remover's
state lock; each context operation fetches the current context. These supplementary
checks are not additional mapped legacy declarations. Native protected `myStatus` has read/write accessor equivalents, with a native derived-class fixture checking raw flag decoding.
