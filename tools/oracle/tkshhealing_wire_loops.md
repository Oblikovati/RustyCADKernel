# Wire loop incidence oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkshhealing_wire_loops.cxx` exercises empty/single wires, chains, branching
vertices, repeated edges, opposite-orientation seam pairs, degenerate edges,
closed edges without geometry, missing vertices and prepopulated output maps.
Each case invokes `CheckLoop` twice with the same maps. Rows record return value,
paired last/accumulated DONE1/FAIL1/FAIL2 flags, infinite precision, map sizes,
indexed loop-vertex order, per-vertex incident-list lengths, and reset status.

The missing-vertex exit deliberately retains native infinite precision and
partial output mutations. The small closed edge without geometry deliberately
retains the native combined DONE1/FAIL1 last status without a loop result.
These fixtures supplement legacy tests; they are not additional legacy declarations.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_loops.cxx -L"$native_build/lin64/gcc/libi" \
  -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/api-proof/tkshhealing-wire-loops
target/api-proof/tkshhealing-wire-loops
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_loops
```
