# Closed-edge division oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The repository-owned C++ fixture is independent of legacy GoogleTest mappings.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_closed_edge_divide.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/rustycad-closed-edge-oracle
/tmp/rustycad-closed-edge-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_closed_edge_divide
```

Rows contain the case, Compute result, HasCurve3d/HasCurve2d flags, then each
knot sequence's length and values. Rust compares the numeric values by their
binary64 bits. Sampling intentionally uses 22 repeated additions and includes
the final endpoint; the selected circle parameter is one ULP below pi.

Cases cover free 3D curves, simultaneous 3D/2D discovery, tolerance rejection
and clamping, p-curve-only geometry, degenerate edges and open edges. Additional
Rust assertions verify dynamic EdgeDivider dispatch, parent state projection,
parameter-handle sharing, and Clear preserving previous parameter handles.
These fixtures are not exhaustive behavioral proof.
