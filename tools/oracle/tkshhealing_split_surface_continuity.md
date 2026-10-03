# ShapeUpgrade_SplitSurfaceContinuity oracle

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Each row records the
three status flags and U/V split sequences. The suite exercises both knot directions,
successful and rejected removal, pre-existing ranges, `Segment=false` bounds updates,
trim recursion, and offset criterion escalation. `u_default` records the constructor's
native `myCont == 0` initialization: it sets DONE2 from C0 surface continuity but does not
correct or split the interior knot until `SetCriterion(C1)` is called. The last column is
the swept basis curve's multiplicity at parameter 0.5 (-1 for non-swept surfaces); it
shows that linear extrusion replaces its basis after correction while revolution retains it.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_split_surface_continuity.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_split_surface_continuity
/tmp/tkshhealing_split_surface_continuity
cargo test -p rustycad-tkshhealing --test shape_upgrade_split_surface_continuity
```

The probe output is stored in `tkshhealing_split_surface_continuity.expected`. These
fixtures verify bounded cases, not full behavioral equivalence.
