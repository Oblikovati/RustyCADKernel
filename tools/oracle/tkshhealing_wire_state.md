# ShapeFix_Wire state and analyzer delegation

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The native fixture records 11 rows for constructor defaults, virtual precision
propagation, mutable closed mode, wire/face construction and initialization,
shared-analyzer initialization, and surface setter overloads.

Rust compares every numeric field exactly (parsing native decimal spelling) and
checks shared handle identity. The precision regression calls through `dyn FixRoot`
to exercise the derived override: root tolerance limits and analyzer precision
must both change. `Init(analyzer)` deliberately retains root precision until the
next SetPrecision, matching OCCT. The located-surface row uses identity placement;
nonidentity placement remains covered by the underlying analyzer tests, not this
new fixture. These are additional oracle regressions, not legacy test mappings.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_wire_state.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_state
/tmp/tkshhealing_wire_state
cargo test -p rustycad-tkshhealing --test shape_fix_wire_state
```
