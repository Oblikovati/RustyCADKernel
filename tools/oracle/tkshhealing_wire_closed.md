# ShapeFix_Wire closing repair oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkshhealing_wire_closed.cxx` records unloaded state and coincident, near, and far
closing endpoints, with and without a plane face. Each loaded case is called twice
and reloaded to check status reset. Rust compares all 13 output rows, including
per-step DONE/FAIL flags, final LastFixStatus, result, and edge count.

`FixClosed` calls `FixConnected(1, prec)`, `FixDegenerated(1)`, then
`FixLacking(1, false)` regardless of preceding return values. The native final
LastFixStatus belongs to FixLacking, while StatusClosed accumulates all three steps.
The Rust wrapper inherits the documented incomplete bent-pcurve follow-up branch
of FixLacking. Its mapping remains partial until that dependency reaches parity.
These focused oracle cases are additional tests, not legacy declaration mappings.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_wire_closed.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_closed
/tmp/tkshhealing_wire_closed
cargo test -p rustycad-tkshhealing --test shape_fix_wire_closed
```
