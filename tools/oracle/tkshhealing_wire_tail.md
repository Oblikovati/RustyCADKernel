# Tail overlap and edge splitting oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Thirteen native cases exercise `CheckTail`: coincident whole edges, either edge
shorter, reversed inputs, disconnected endpoints, angle rejection and disabled
angle checking, missing geometry, degeneration, a zero-length curve, an unloaded
analyzer, and whole/partial Bézier overlap. The disabled-angle case exercises
bisection and two edge splits. The shorter Bézier case exercises nonlinear
projection and parameter transfer.

Rows contain the return value and four edge parts. Each part records identity
against both inputs and a retained sentinel, orientation, parameter range,
SameParameter/SameRange flags, and both endpoint coordinates and tolerances.
The final flags are last-check OK/DONE1/DONE2/FAIL1/FAIL2, seeded by CheckSmall
before calling CheckTail. CheckTail does not reset or alter these flags.
Rust compares all recorded floating-point values exactly against native output.
These fixtures supplement legacy declarations and do not claim toolkit completion.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_tail.cxx -L"$native_build/lin64/gcc/libi" \
  -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/api-proof/tkshhealing-wire-tail
target/api-proof/tkshhealing-wire-tail
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_tail
```
