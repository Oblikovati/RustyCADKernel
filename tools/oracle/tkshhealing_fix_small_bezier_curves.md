# Small Bézier curve repair oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_fix_small_bezier_curves.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/rustycad-small-bezier-oracle
/tmp/rustycad-small-bezier-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_fix_small_bezier_curves
```

Rows contain name, return value (-1 means an escaping Standard_Failure), resulting
first/last parameters, three output-presence flags, and the tool's OK status.
The named criterion subclasses use real native geometry and splitter Build, then
inject FAIL1 or DONE2, add a split, or throw during Build. The Rust fixtures use
matching subclasses and inspect output geometry as well as the frozen rows.

The fixture distinguishes the native 3D FAIL acceptance from forward p-curve FAIL
rejection and reversed p-curve DONE requirement. A cylindrical face exercises true
seam handling, including resetting First to the forward p-curve's lower bound.
CN continuity throws inside the forward approximation catch but outside the reversed
approximation catch. Equal trim bounds throw before both catch regions. Existing
partial outputs and the tool status survive later failures.

These sixteen generated cases supplement the legacy test inventory. They are not
exhaustive proof for all curve families, approximation-not-done outcomes or native
undefined null-tool dereferences. Four Rust tests cover the rows, geometry and shared
parent state; no legacy declaration mapping is fabricated from these fixtures.
