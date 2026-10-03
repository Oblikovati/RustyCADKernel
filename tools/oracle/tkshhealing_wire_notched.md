# Adjacent notched-edge oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The native fixture measures `CheckNotchedEdges` on cylindrical-face p-curves;
the algorithm projects those p-curves onto a temporary plane, as in OCCT.
Thirteen rows cover shorter-first/second edges, equal-distance tie selection,
reversed edges, degenerate edges, missing/disconnected vertices, missing p-curves,
nonparallel tangents, UV gaps, zero derivatives, curved overlap rejection and an
unloaded analyzer. The curved rejection occurs after the output edge index and
parameter have been written; the Rust test verifies that partial output mutation.
Success leaves the native last status at OK rather than setting DONE.

Columns are name, boolean result, short edge index, projection parameter,
DONE, FAIL1, FAIL2 and FAIL3. All numeric outputs compare exactly in this fixture.
These probes supplement, and do not increase the count of, mapped legacy tests.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_notched.cxx -L"$native_build/lin64/gcc/libi" \
  -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/api-proof/tkshhealing-wire-notched
target/api-proof/tkshhealing-wire-notched
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_notched
```
